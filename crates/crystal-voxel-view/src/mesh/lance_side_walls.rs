//! Fold Lance's authenticated blue edge courses into continuous closed walls.
//! The source wall quadrants remain nonwalkable; scenery never changes support.
use super::*;

fn source_drawing(block: u16) -> Option<[[u16; 4]; 4]> {
    drawing(block).or_else(|| {
        Some(match block {
            0x38 => [
                [129, 93, 149, 151],
                [130, 93, 93, 151],
                [131, 93, 93, 149],
                [128, 93, 149, 151],
            ],
            0x39 => [
                [151, 150, 93, 133],
                [151, 93, 93, 134],
                [150, 93, 93, 135],
                [151, 150, 93, 132],
            ],
            0x14 => [
                [129, 93, 149, 151],
                [130, 93, 93, 93],
                [37, 38, 38, 39],
                [88, 89, 88, 89],
            ],
            0x37 => [
                [151, 150, 93, 133],
                [93, 93, 93, 134],
                [37, 38, 38, 39],
                [88, 89, 88, 89],
            ],
            _ => return None,
        })
    })
}

fn expected(x: usize, y: usize) -> u16 {
    match (x < 4, y / 4) {
        (true, 1) => 0x2f,
        (false, 1) => 0x30,
        (true, 2) => 0x3a,
        (false, 2) => 0x3b,
        (true, 11) => 0x14,
        (false, 11) => 0x37,
        (true, _) => 0x38,
        (false, _) => 0x39,
    }
}

pub(super) fn complete(
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
    profiles: Option<&crate::live_profiles::Document>,
) -> bool {
    if !super::complete(frame, cells, g)
        || frame.map_id.as_ref() != "LancesRoom"
        || frame.source_map_size_core_tiles != Some(UVec2::new(10, 24))
        || cells.len() != g.width * g.height
    {
        return false;
    }
    let origin = frame.grid_origin;
    for y in 4..48 {
        for x in (0..4).chain(16..20) {
            let (lx, ly) = (x - origin.x, y - origin.y);
            if lx < 0 || ly < 0 || lx as usize >= g.width || ly as usize >= g.height {
                return false;
            }
            let s = &cells[ly as usize * g.width + lx as usize].source;
            let b = expected(x as usize, y as usize);
            if s.tileset_id.as_ref() != "champions_room"
                || s.metatile_id != b
                || s.subtile_column != x as u8 % 4
                || s.subtile_row != y as u8 % 4
                || source_drawing(b).unwrap()[y as usize % 4][x as usize % 4] != s.tile_index
            {
                return false;
            }
        }
    }
    !live::resolve(cells, g.width, g.height, "LancesRoom", profiles)
        .iter()
        .any(|p| {
            p.indices(g.width).any(|i| {
                let x = origin.x + (i % g.width) as i32;
                let y = origin.y + (i / g.width) as i32;
                ((0..4).contains(&x) || (16..20).contains(&x)) && (4..48).contains(&y)
            })
        })
}

pub(super) fn finish(
    mesh: &mut TerrainMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
    profiles: Option<&crate::live_profiles::Document>,
) {
    if !complete(frame, cells, g, profiles) {
        return;
    }
    let origin = frame.grid_origin;
    let scale = g.tile_height / 8.;
    // Fold both columns of the source wall: its outer edge and brick face.
    // The gray floor, every statue and the south door retain their geometry.
    for surface in [
        &mut mesh.textured,
        &mut mesh.solid,
        &mut mesh.animated_textured,
        &mut mesh.animated_solid,
    ] {
        let old = std::mem::take(&mut surface.indices);
        for tri in old.chunks_exact(3) {
            let p = tri
                .iter()
                .map(|&i| Vec3::from_array(surface.positions[i as usize]))
                .sum::<Vec3>()
                / 3.;
            let x = (p.x - g.origin_x) / g.tile_width + origin.x as f32;
            let z = (p.z - g.origin_z) / g.tile_height + origin.y as f32;
            let base = if z < 12. { 16. } else { 0. };
            let fold = ((0.0..2.0).contains(&x) || (18.0..20.0).contains(&x))
                && (4.0..46.0).contains(&z)
                && (p.y - base * scale).abs() < 0.001 * scale
                && tri.iter().all(|&i| surface.normals[i as usize][1] > 0.99);
            if !fold {
                surface.indices.extend_from_slice(tri);
            }
        }
    }
    let out = &mut mesh.textured;
    for west in [true, false] {
        let start = out.positions.len();
        for row in 4..46 {
            let column = if west { 0 } else { 19 };
            let (lx, ly) = ((column - origin.x) as usize, (row - origin.y) as usize);
            let (mut w, mut e, n, s) = g.bounds(lx, ly);
            if west {
                e += g.tile_width;
            } else {
                w -= g.tile_width;
            }
            let bottom = if row < 12 { 16. * scale } else { 0. };
            let top = 48. * scale;
            let face_column = if west { lx + 1 } else { lx - 1 };
            debug_assert_eq!(cells[ly * g.width + face_column].source.tile_index, 93);
            let uv = g.uv(face_column, ly);
            // The authenticated north-course field paint makes a continuous
            // cap and back. Edge glyphs must never become giant wall stripes.
            let cap = g.uv((3 - origin.x) as usize, (-origin.y) as usize);
            let paint = [(cap.0 + cap.1) * 0.5, (cap.2 + cap.3) * 0.5];
            // Each eight-pixel face course retains native texel scale rather
            // than stretching the original eight pixels up the entire wall.
            let mut y = bottom;
            while y < top - 0.001 * scale {
                let h = (y + 8. * scale).min(top);
                let points = if west {
                    [[e, y, s], [e, y, n], [e, h, n], [e, h, s]]
                } else {
                    [[w, y, n], [w, y, s], [w, h, s], [w, h, n]]
                };
                append_quad(
                    out,
                    points,
                    [if west { 1. } else { -1. }, 0., 0.],
                    [[uv.0, uv.3], [uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2]],
                    TEXTURED_SHADE,
                );
                y = h;
            }
            for (points, normal) in [
                (
                    [[w, top, n], [w, top, s], [e, top, s], [e, top, n]],
                    [0., 1., 0.],
                ),
                (
                    [
                        [w, bottom, n],
                        [e, bottom, n],
                        [e, bottom, s],
                        [w, bottom, s],
                    ],
                    [0., -1., 0.],
                ),
                (
                    [[w, bottom, n], [w, bottom, s], [w, top, s], [w, top, n]],
                    [-1., 0., 0.],
                ),
                (
                    [[e, bottom, s], [e, bottom, n], [e, top, n], [e, top, s]],
                    [1., 0., 0.],
                ),
                (
                    [[e, bottom, n], [w, bottom, n], [w, top, n], [e, top, n]],
                    [0., 0., -1.],
                ),
                (
                    [[w, bottom, s], [e, bottom, s], [e, top, s], [w, top, s]],
                    [0., 0., 1.],
                ),
            ] {
                // The subdivided interior replaces this box face exactly.
                if normal[0] == if west { 1. } else { -1. } {
                    continue;
                }
                append_quad(out, points, normal, [paint; 4], TEXTURED_SHADE);
            }
            if mesh.authored_cells.len() == cells.len() {
                for x in [lx, face_column] {
                    mesh.authored_cells[ly * g.width + x] = Some("lance:blue-side-wall");
                }
            }
        }
        out.cutaway_ranges.push(start..out.positions.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (VisualWorldFrame, GridGeometry, TerrainMeshData) {
        let pack = crystal_assets::read_loaded_verified_compiled_game_pack(
            std::path::PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap()),
        )
        .unwrap();
        let data = &pack.pack().data().maps["LancesRoom"];
        let art = &pack.pack().runtime_files()["data/tilesets/champions_room_metatiles.bin"];
        let tiles = (0..960)
            .map(|i| {
                let (x, y) = (i % 20, i / 20);
                let b = data.blocks[y / 4 * 5 + x / 4];
                VisualTileSource {
                    tileset_id: "champions_room".into(),
                    metatile_id: b,
                    subtile_column: (x % 4) as u8,
                    subtile_row: (y % 4) as u8,
                    tile_index: art[b as usize * 16 + y % 4 * 4 + x % 4] as u16,
                }
            })
            .collect();
        let mut frame = super::super::super::tests::frame(20, 48, tiles);
        frame.map_id = "LancesRoom".into();
        frame.source_map_size_core_tiles = Some(UVec2::new(10, 24));
        let g = GridGeometry {
            width: 20,
            height: 48,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut mesh = TerrainMeshData {
            footing_heights: vec![2.75; 960],
            authored_cells: vec![None; 960],
            ..Default::default()
        };
        for y in 0..48 {
            for x in 0..20 {
                append_top(
                    &mut mesh.textured,
                    g.bounds(x, y).into(),
                    if y < 12 { 16. } else { 0. },
                    g.uv(x, y),
                );
            }
        }
        (frame, g, mesh)
    }
    #[test]
    fn external_lance_blue_walls_are_closed_contiguous_and_keep_every_walking_height() {
        let (frame, g, baseline) = fixture();
        let source = frame.clone();
        let mut mesh = baseline.clone();
        finish(
            &mut mesh,
            &frame,
            &frame.tiles.iter().collect::<Vec<_>>(),
            &g,
            None,
        );
        assert_eq!(frame, source);
        assert_eq!(mesh.footing_heights, baseline.footing_heights);
        assert_eq!(
            mesh.authored_cells
                .iter()
                .filter(|&&v| v == Some("lance:blue-side-wall"))
                .count(),
            168
        );
        assert_eq!(mesh.textured.cutaway_ranges.len(), 2);
        for range in &mesh.textured.cutaway_ranges {
            let mut volume = 0.0;
            for tri in mesh
                .textured
                .indices
                .chunks_exact(3)
                .filter(|t| t.iter().all(|&i| range.contains(&(i as usize))))
            {
                let p = [tri[0], tri[1], tri[2]]
                    .map(|i| Vec3::from_array(mesh.textured.positions[i as usize]));
                let n = Vec3::from_array(mesh.textured.normals[tri[0] as usize]);
                assert!((p[1] - p[0]).cross(p[2] - p[0]).dot(n) > 0.);
                for p in p {
                    assert!(p.y <= 48. && p.y >= 0.);
                }
                volume += p[0].dot(p[1].cross(p[2])) / 6.;
            }
            assert!(
                (volume - 128. * (8. * 32. + 34. * 48.)).abs() < 1.,
                "closed wall sections include their backs, soles and both ends"
            );
            for z in 4..46 {
                for y in 0..48 {
                    if z < 12 && y < 16 {
                        continue;
                    }
                    let x = if range.start == mesh.textured.cutaway_ranges[0].start {
                        16.
                    } else {
                        144.
                    };
                    let point = Vec2::new(z as f32 * 8. + 4., y as f32 + 0.5);
                    assert!(
                        mesh.textured.indices.chunks_exact(3).any(|t| {
                            if !t.iter().all(|&i| range.contains(&(i as usize))) {
                                return false;
                            }
                            let p = [t[0], t[1], t[2]]
                                .map(|i| Vec3::from_array(mesh.textured.positions[i as usize]));
                            if !p.iter().all(|p| (p.x - x).abs() < 0.001) {
                                return false;
                            }
                            let p = p.map(|p| Vec2::new(p.z, p.y));
                            let cross = |a: Vec2, b: Vec2| a.x * b.y - a.y * b.x;
                            let a = [
                                cross(p[1] - p[0], point - p[0]),
                                cross(p[2] - p[1], point - p[1]),
                                cross(p[0] - p[2], point - p[2]),
                            ];
                            a.iter().all(|&v| v >= -0.001) || a.iter().all(|&v| v <= 0.001)
                        }),
                        "gap at longitudinal source cell {z}, height {y}"
                    );
                }
            }
        }
    }
    #[test]
    fn external_lance_blue_walls_reject_edited_clipped_and_custom_source_courses() {
        let (mut frame, g, baseline) = fixture();
        for i in [4 * 20, 7 * 20 + 3, 20 * 20 + 19, 47 * 20 + 16] {
            frame.tiles[i].source.tile_index ^= 1;
            let mut mesh = baseline.clone();
            finish(
                &mut mesh,
                &frame,
                &frame.tiles.iter().collect::<Vec<_>>(),
                &g,
                None,
            );
            assert_eq!(mesh, baseline);
            frame.tiles[i].source.tile_index ^= 1;
        }
        let custom:crate::live_profiles::Document=serde_json::from_str(r#"{"objects":[{"name":"Custom wall","map":"LancesRoom","tileset":"champions_room","metatile":56,"origin":[0,0],"tiles":[[129]],"ground":93,"top_pixels":0,"depth_pixels":8}]}"#).unwrap();
        let mut mesh = baseline.clone();
        finish(
            &mut mesh,
            &frame,
            &frame.tiles.iter().collect::<Vec<_>>(),
            &g,
            Some(&custom),
        );
        assert_eq!(mesh, baseline);
        frame.grid_origin.y = 1;
        let mut mesh = baseline.clone();
        finish(
            &mut mesh,
            &frame,
            &frame.tiles.iter().collect::<Vec<_>>(),
            &g,
            None,
        );
        assert_eq!(mesh, baseline);
    }
}
