//! Fold the script-owned champion entrance drawing into a closed south course.
//! The current metatile selects the barrier; collision and warps remain source-owned.
use super::*;

fn drawing(block: u16) -> Option<[[u16; 4]; 4]> {
    Some(match block {
        0x14 => [
            [129, 93, 149, 151],
            [130, 93, 93, 93],
            [37, 38, 38, 39],
            [88, 89, 88, 89],
        ],
        0x2b => [[151; 4], [93; 4], [37, 38, 38, 39], [88, 89, 88, 89]],
        0x31 => [[59, 60, 60, 61]; 4],
        0x34 => [
            [59, 60, 60, 61],
            [59, 60, 60, 61],
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
}

pub(super) fn complete(
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
    profiles: Option<&crate::live_profiles::Document>,
) -> bool {
    if frame.map_id.as_ref() != "LancesRoom"
        || frame.source_map_size_core_tiles != Some(UVec2::new(10, 24))
        || cells.len() != g.width * g.height
    {
        return false;
    }
    let o = frame.grid_origin;
    for y in 44..48 {
        for x in 0..20 {
            let (lx, ly) = (x - o.x, y - o.y);
            if lx < 0 || ly < 0 || lx as usize >= g.width || ly as usize >= g.height {
                return false;
            }
            let s = &cells[ly as usize * g.width + lx as usize].source;
            let valid = match x / 4 {
                0 => s.metatile_id == 0x14,
                1 | 3 => s.metatile_id == 0x2b,
                2 => matches!(s.metatile_id, 0x31 | 0x34),
                4 => s.metatile_id == 0x37,
                _ => false,
            };
            if !valid
                || s.tileset_id.as_ref() != "champions_room"
                || s.subtile_column != x as u8 % 4
                || s.subtile_row != y as u8 % 4
                || drawing(s.metatile_id)
                    .is_none_or(|d| d[y as usize % 4][x as usize % 4] != s.tile_index)
            {
                return false;
            }
            // A block switch must be atomic; accepting mixed open/closed rows
            // would turn a partially customized drawing into an invented door.
            let anchor = &cells[(44 - o.y) as usize * g.width + (x / 4 * 4 - o.x) as usize].source;
            if s.metatile_id != anchor.metatile_id {
                return false;
            }
        }
    }
    !live::resolve(cells, g.width, g.height, "LancesRoom", profiles)
        .iter()
        .any(|p| {
            p.indices(g.width).any(|i| {
                (0..20).contains(&(o.x + (i % g.width) as i32))
                    && (44..48).contains(&(o.y + (i / g.width) as i32))
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
    let o = frame.grid_origin;
    let closed = cells[(44 - o.y) as usize * g.width + (8 - o.x) as usize]
        .source
        .metatile_id
        == 0x34;
    let occupied = |x: f32| (0.0..20.0).contains(&x) && (closed || !(8.0..12.0).contains(&x));
    let at = |x: f32, y: f32, z: f32| {
        [
            g.origin_x + (x - o.x as f32) * g.tile_width,
            y * g.tile_height / 8.,
            g.origin_z + (z - o.y as f32) * g.tile_height,
        ]
    };
    // Only folded horizontal wall artwork is retired. The carpet, source
    // warp lanes, walking datum and neighboring statue geometry survive.
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
            let x = (p.x - g.origin_x) / g.tile_width + o.x as f32;
            let z = (p.z - g.origin_z) / g.tile_height + o.y as f32;
            let flat = occupied(x)
                && (46.0..48.0).contains(&z)
                && p.y.abs() < 0.001 * g.tile_height
                && tri.iter().all(|&i| surface.normals[i as usize][1] > 0.99);
            if !flat {
                surface.indices.extend_from_slice(tri);
            }
        }
    }
    let paint_uv = g.uv((2 - o.x) as usize, (45 - o.y) as usize);
    let paint = [
        (paint_uv.0 + paint_uv.1) * 0.5,
        (paint_uv.2 + paint_uv.3) * 0.5,
    ];
    let sections: &[(usize, usize)] = if closed {
        &[(0, 20)]
    } else {
        &[(0, 8), (12, 20)]
    };
    for &(west, east) in sections {
        let out = &mut mesh.textured;
        let first = out.positions.len();
        // Every exposed face is closed, including the ends of the open doorway.
        for (points, normal) in [
            (
                [
                    at(west as f32, 48., 46.),
                    at(west as f32, 48., 48.),
                    at(east as f32, 48., 48.),
                    at(east as f32, 48., 46.),
                ],
                [0., 1., 0.],
            ),
            (
                [
                    at(west as f32, 0., 48.),
                    at(west as f32, 0., 46.),
                    at(east as f32, 0., 46.),
                    at(east as f32, 0., 48.),
                ],
                [0., -1., 0.],
            ),
            (
                [
                    at(west as f32, 0., 48.),
                    at(east as f32, 0., 48.),
                    at(east as f32, 48., 48.),
                    at(west as f32, 48., 48.),
                ],
                [0., 0., 1.],
            ),
            (
                [
                    at(west as f32, 0., 46.),
                    at(west as f32, 0., 48.),
                    at(west as f32, 48., 48.),
                    at(west as f32, 48., 46.),
                ],
                [-1., 0., 0.],
            ),
            (
                [
                    at(east as f32, 0., 48.),
                    at(east as f32, 0., 46.),
                    at(east as f32, 48., 46.),
                    at(east as f32, 48., 48.),
                ],
                [1., 0., 0.],
            ),
        ] {
            append_quad(out, points, normal, [paint; 4], TEXTURED_SHADE);
        }
        for x in west..east {
            for (bottom, top, row) in [(0., 8., Some(47)), (8., 40., None), (40., 48., Some(46))] {
                let uv = if let Some(row) = row {
                    let u = g.uv((x as i32 - o.x) as usize, (row - o.y) as usize);
                    [[u.1, u.3], [u.0, u.3], [u.0, u.2], [u.1, u.2]]
                } else {
                    [paint; 4]
                };
                append_quad(
                    out,
                    [
                        at(x as f32 + 1., bottom, 46.),
                        at(x as f32, bottom, 46.),
                        at(x as f32, top, 46.),
                        at(x as f32 + 1., top, 46.),
                    ],
                    [0., 0., -1.],
                    uv,
                    TEXTURED_SHADE,
                );
            }
            if mesh.authored_cells.len() == cells.len() {
                for y in 46..48 {
                    mesh.authored_cells[(y - o.y) as usize * g.width + (x as i32 - o.x) as usize] =
                        Some("lance:south-barrier");
                }
            }
        }
        let range = first..out.positions.len();
        out.cutaway_ranges.push(range.clone());
        mesh.camera_wall_textured_ranges
            .push((range, [0., -1., -at(0., 0., 46.)[2]]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(
        scale: f32,
        center: u16,
        north: u16,
    ) -> (VisualWorldFrame, GridGeometry, TerrainMeshData) {
        let pack = crystal_assets::read_loaded_verified_compiled_game_pack(
            std::path::PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap()),
        )
        .unwrap();
        let data = &pack.pack().data().maps["LancesRoom"];
        let art = &pack.pack().runtime_files()["data/tilesets/champions_room_metatiles.bin"];
        let tiles = (0..960)
            .map(|i| {
                let (x, y) = (i % 20, i / 20);
                let b = if (8..12).contains(&x) && y >= 44 {
                    center
                } else if (8..12).contains(&x) && y < 4 {
                    north
                } else {
                    data.blocks[y / 4 * 5 + x / 4]
                };
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
            tile_width: scale,
            tile_height: scale,
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
                append_top(&mut mesh.textured, g.bounds(x, y).into(), 0., g.uv(x, y));
            }
        }
        (frame, g, mesh)
    }
    #[test]
    fn external_lance_south_barrier_is_closed_to_native_ceiling_and_follows_every_door_drawing() {
        for scale in [8., 32.] {
            for center in [0x31, 0x34] {
                for north in [0x2e, 0x0b] {
                    let (frame, g, mut mesh) = fixture(scale, center, north);
                    let baseline = mesh.clone();
                    let source = frame.clone();
                    let refs = frame.tiles.iter().collect::<Vec<_>>();
                    assert!(super::super::owns_perimeter(&frame, &refs, &g, None));
                    finish(&mut mesh, &frame, &refs, &g, None);
                    assert_eq!(frame, source);
                    assert_eq!(mesh.footing_heights, baseline.footing_heights);
                    let width = if center == 0x34 { 20 } else { 16 };
                    assert_eq!(
                        mesh.authored_cells
                            .iter()
                            .filter(|&&x| x == Some("lance:south-barrier"))
                            .count(),
                        width * 2
                    );
                    assert_eq!(
                        mesh.textured.cutaway_ranges.len(),
                        if center == 0x34 { 1 } else { 2 }
                    );
                    assert_eq!(
                        mesh.camera_wall_textured_ranges.len(),
                        mesh.textured.cutaway_ranges.len()
                    );
                    let mut volume = 0.;
                    for (range, plane) in &mesh.camera_wall_textured_ranges {
                        assert_eq!(*plane, [0., -1., -46. * scale]);
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
                            volume += p[0].dot(p[1].cross(p[2])) / 6.;
                            if center != 0x34 {
                                assert!(
                                    !p.iter().any(|v| v.x > 8. * scale && v.x < 12. * scale),
                                    "both entrance lanes stay open"
                                );
                            }
                        }
                    }
                    let expected = width as f32 * scale * 2. * scale * 6. * scale;
                    assert!(
                        (volume - expected).abs() < expected * 0.0001,
                        "every section has a sole, cap, back and closed ends"
                    );
                    for x in 0..20 {
                        for y in 0..48 {
                            let point = bevy::prelude::Vec2::new(
                                (x as f32 + 0.5) * scale,
                                (y as f32 + 0.5) * scale / 8.,
                            );
                            let covered = mesh.textured.indices.chunks_exact(3).any(|t| {
                                let p = [t[0], t[1], t[2]]
                                    .map(|i| Vec3::from_array(mesh.textured.positions[i as usize]));
                                if !p.iter().all(|v| (v.z - 46. * scale).abs() < 0.001 * scale) {
                                    return false;
                                }
                                let p = p.map(|v| bevy::prelude::Vec2::new(v.x, v.y));
                                let cross = |a: bevy::prelude::Vec2, b: bevy::prelude::Vec2| {
                                    a.x * b.y - a.y * b.x
                                };
                                if cross(p[1] - p[0], p[2] - p[0]).abs() < 0.001 {
                                    return false;
                                }
                                let s = [
                                    cross(p[1] - p[0], point - p[0]),
                                    cross(p[2] - p[1], point - p[1]),
                                    cross(p[0] - p[2], point - p[2]),
                                ];
                                s.iter().all(|&v| v >= -0.001) || s.iter().all(|&v| v <= 0.001)
                            });
                            assert_eq!(
                                covered,
                                center == 0x34 || !(8..12).contains(&x),
                                "source/native face x{x} height{y}"
                            );
                        }
                    }
                    // The untouched open passage keeps exactly its original floor quads.
                    if center != 0x34 {
                        for i in (46 * 20..48 * 20).filter(|i| (8..12).contains(&(i % 20))) {
                            assert!(
                                mesh.textured.indices.chunks_exact(3).any(
                                    |t| t == [i as u32 * 4, i as u32 * 4 + 1, i as u32 * 4 + 2]
                                )
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn external_lance_south_barrier_rejects_each_source_edit_and_custom_ownership_atomically() {
        let (mut frame, g, baseline) = fixture(8., 0x34, 0x2e);
        for y in 44..48 {
            for x in 0..20 {
                let i = y * 20 + x;
                frame.tiles[i].source.tile_index ^= 1;
                let refs = frame.tiles.iter().collect::<Vec<_>>();
                let mut mesh = baseline.clone();
                finish(&mut mesh, &frame, &refs, &g, None);
                assert_eq!(mesh, baseline);
                assert!(!super::super::owns_perimeter(&frame, &refs, &g, None));
                frame.tiles[i].source.tile_index ^= 1;
            }
        }
        let custom:crate::live_profiles::Document=serde_json::from_str(r#"{"objects":[{"name":"Custom entrance","map":"LancesRoom","tileset":"champions_room","metatile":43,"origin":[0,0],"tiles":[[151]],"ground":93,"top_pixels":0,"depth_pixels":8}]}"#).unwrap();
        let refs = frame.tiles.iter().collect::<Vec<_>>();
        assert!(!live::resolve(&refs, g.width, g.height, "LancesRoom", Some(&custom)).is_empty());
        let mut mesh = baseline.clone();
        finish(&mut mesh, &frame, &refs, &g, Some(&custom));
        assert_eq!(mesh, baseline);
        assert!(!super::super::owns_perimeter(
            &frame,
            &refs,
            &g,
            Some(&custom)
        ));
        // $33 has a blocked north collision quadrant despite its plain art.
        // It is not an authored entrance alternative; never invent an open
        // passage from that drawing or suppress its fallback enclosure.
        let (unsupported, g, baseline) = fixture(8., 0x33, 0x2e);
        let refs = unsupported.tiles.iter().collect::<Vec<_>>();
        let mut untouched = baseline.clone();
        finish(&mut untouched, &unsupported, &refs, &g, None);
        assert_eq!(untouched, baseline);
        assert!(!super::super::owns_perimeter(&unsupported, &refs, &g, None));
        // A valid-looking tile from another source state is still a partial block edit.
        frame.tiles[44 * 20 + 8].source.metatile_id = 0x31;
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
