//! The leader alcove has inset side walls; its cap art is not floating trim.
use super::*;
use bevy::prelude::Vec2;

const HEADER: [[u16; 5]; 3] = [
    [0x22, 0x26, 0x26, 0x26, 0x24],
    [0x23, 0x01, 0x01, 0x01, 0x25],
    [0x20, 0x21, 0x01, 0x20, 0x21],
];
fn drawing(block: u16) -> [[u16; 4]; 4] {
    match block {
        0x01 => [
            [1, 0x10, 1, 0x10],
            [0x11, 0x12, 0x11, 0x12],
            [1, 0x10, 1, 0x10],
            [0x11, 0x12, 0x11, 0x12],
        ],
        0x20 => [
            [0x48, 0x49, 0x49, 0x49],
            [0x58, 0x59, 0x59, 0x59],
            [0x5c, 0x4d, 0x4d, 0x4d],
            [0x5e, 0x5b, 0x5b, 0x5b],
        ],
        0x21 => [
            [0x49, 0x49, 0x49, 0x4a],
            [0x59, 0x59, 0x59, 0x5a],
            [0x4d, 0x4d, 0x4d, 0x5d],
            [0x5b, 0x5b, 0x5b, 0x5f],
        ],
        0x22 => [
            [0x49, 0x49, 0x49, 0x4a],
            [0x2d, 0x2d, 0x2d, 0x4c],
            [0x2d, 0x2d, 0x2d, 0x4c],
            [0x2d, 0x2d, 0x2d, 0x4c],
        ],
        0x23 => [[0x2d, 0x2d, 0x2d, 0x4c]; 4],
        0x24 => [
            [0x48, 0x49, 0x49, 0x49],
            [0x4b, 0x2d, 0x2d, 0x2d],
            [0x4b, 0x2d, 0x2d, 0x2d],
            [0x4b, 0x2d, 0x2d, 0x2d],
        ],
        0x25 => [[0x4b, 0x2d, 0x2d, 0x2d]; 4],
        0x26 => [
            [0x4d; 4],
            [0x5b; 4],
            [1, 0x10, 1, 0x10],
            [0x11, 0x12, 0x11, 0x12],
        ],
        _ => unreachable!("authenticated gym header"),
    }
}
fn header_wall(x: i32, y: i32) -> bool {
    (0..20).contains(&x) && (0..8).contains(&y) && (x < 4 || x >= 16 || y < 2)
}
fn topology(
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
    profiles: Option<&Document>,
) -> bool {
    if frame.map_id.as_ref() != "VermilionGym"
        || frame.source_map_size_core_tiles != Some(bevy::math::UVec2::new(10, 18))
        || g.width.checked_mul(g.height) != Some(cells.len())
    {
        return false;
    }
    for y in 0..36i32 {
        for x in 0..20i32 {
            let (c, r) = (x - frame.grid_origin.x, y - frame.grid_origin.y);
            if c < 0 || r < 0 || c as usize >= g.width || r as usize >= g.height {
                return false;
            }
            let s = &cells[r as usize * g.width + c as usize].source;
            if s.tileset_id.as_ref() != "game_corner"
                || i32::from(s.subtile_column) != x % 4
                || i32::from(s.subtile_row) != y % 4
            {
                return false;
            }
            if y < 12 {
                let block = HEADER[y as usize / 4][x as usize / 4];
                if s.metatile_id != block
                    || s.tile_index != drawing(block)[y as usize % 4][x as usize % 4]
                {
                    return false;
                }
            } else if s.tile_index == 0x2d {
                return false;
            }
        }
    }
    if frame.boundary_warps.len() != 2
        || ![IVec2::new(4, 17), IVec2::new(5, 17)]
            .iter()
            .all(|w| frame.boundary_warps.contains(w))
    {
        return false;
    }
    if let Some(profiles) = profiles {
        let canonical = built_in_profiles();
        let mut objects = Vec::new();
        for custom in &profiles.objects {
            if canonical.objects.contains(custom) {
                continue;
            }
            objects.push(custom.clone());
            if let Some(old) = canonical.objects.iter().find(|o| o.name == custom.name) {
                objects.push(old.clone());
            }
        }
        let custom = Document {
            objects,
            atmosphere: None,
        };
        if live::resolve(cells, g.width, g.height, &frame.map_id, Some(&custom))
            .iter()
            .any(|p| {
                p.indices(g.width).any(|i| {
                    let x = frame.grid_origin.x + (i % g.width) as i32;
                    let y = frame.grid_origin.y + (i / g.width) as i32;
                    (0..20).contains(&x)
                        && (0..36).contains(&y)
                        && (y < 12 || y >= 34 || x < 2 || x >= 18)
                })
            })
        {
            return false;
        }
    }
    true
}

/// Borrow the rear course's blue field and light cap, not a partition's black panel.
pub(super) fn template(
    out: &mut TerrainMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
    profiles: Option<&Document>,
) -> Option<usize> {
    if !topology(frame, cells, g, profiles) {
        return None;
    }
    let column = (4 - frame.grid_origin.x) as usize;
    let row = (-frame.grid_origin.y) as usize;
    let sample = |column: usize, row: usize, x: f32, y: f32| {
        let uv = g.uv(column, row);
        [
            uv.0 + (uv.1 - uv.0) * (x + 0.5) / 8.,
            uv.2 + (uv.3 - uv.2) * (y + 0.5) / 8.,
        ]
    };
    let blue = sample(column, row, 3., 3.);
    let trim = sample((-frame.grid_origin.x) as usize, row, 0., 1.);
    let (w, e, _, _) = g.bounds(column, row);
    let e = e + g.tile_width;
    let front = g.origin_z + (row + 2) as f32 * g.tile_height;
    let unit = g.tile_height / 8.;
    for (low, high, north, south, paint) in [
        (0., 2., front - 3. * unit, front, trim),
        (2., 31., front - 3. * unit, front, blue),
        (31., 32., front - 3. * unit, front, trim),
    ] {
        let start = out.textured.positions.len();
        architecture_box(
            &mut out.textured,
            [w, e, low * unit, high * unit, north, south],
            [1.; 3],
        );
        out.textured.uvs[start..].fill(paint);
    }
    Some(row)
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
    profiles: Option<&Document>,
    solid: &SurfaceMeshData,
    textured: &SurfaceMeshData,
    b: [f32; 4],
) {
    if !topology(frame, cells, g, profiles) {
        return;
    }
    // Remove only the authenticated source header's old flat/folded art. Its
    // neighboring lawn/floor samples and the complete central panels survive.
    for (surface, source_uv) in [
        (&mut mesh.textured, true),
        (&mut mesh.animated_textured, true),
        (&mut mesh.solid, false),
        (&mut mesh.animated_solid, false),
    ] {
        surface.indices = surface
            .indices
            .chunks_exact(3)
            .filter(|tri| {
                let empty_floor = [
                    (0., 4., 0., 12.),
                    (16., 20., 0., 12.),
                    (0., 8., 8., 12.),
                    (12., 20., 8., 12.),
                ]
                .into_iter()
                .any(|(west, east, north, south)| {
                    tri.iter().all(|&i| {
                        let p = surface.positions[i as usize];
                        let x = frame.grid_origin.x as f32 + (p[0] - g.origin_x) / g.tile_width;
                        let y = frame.grid_origin.y as f32 + (p[2] - g.origin_z) / g.tile_height;
                        p[1].abs() < 0.00001
                            && surface.normals[i as usize][1] > 0.9
                            && x >= west - 0.001
                            && x <= east + 0.001
                            && y >= north - 0.001
                            && y <= south + 0.001
                    })
                });
                if empty_floor {
                    return false;
                }
                if !source_uv {
                    return true;
                }
                let uv = tri
                    .iter()
                    .map(|&i| Vec2::from_array(surface.uvs[i as usize]))
                    .sum::<Vec2>()
                    / 3.;
                let x = frame.grid_origin.x + (uv.x * g.width as f32).floor() as i32;
                let y = frame.grid_origin.y + (uv.y * g.height as f32).floor() as i32;
                !header_wall(x, y)
            })
            .flatten()
            .copied()
            .collect();
    }
    // The partition's four source rows depict its upright face. Continue the
    // actual checker behind its thin native body only inside the inset room.
    for y in 8..12i32 {
        for x in (4..8i32).chain(12..16) {
            let (column, row) = (
                (x - frame.grid_origin.x) as usize,
                (y - frame.grid_origin.y) as usize,
            );
            let (donor_x, donor_y) = (
                (8 + x % 2 - frame.grid_origin.x) as usize,
                (4 + y % 2 - frame.grid_origin.y) as usize,
            );
            crate::mesh::append_top(
                &mut mesh.textured,
                g.bounds(column, row).into(),
                0.,
                g.uv(donor_x, donor_y),
            );
        }
    }
    let west = g.origin_x - frame.grid_origin.x as f32 * g.tile_width;
    let north = g.origin_z - frame.grid_origin.y as f32 * g.tile_height;
    let unit = g.tile_height / 8.;
    let thickness = b[3] - b[2];
    let join = north + 12. * g.tile_height - 6. * unit; // exact back of the native threshold panels
    for (edge, at, from, to, step) in [
        (
            Edge::North,
            north + 2. * g.tile_height,
            west + 4. * g.tile_width - thickness,
            west + 16. * g.tile_width + thickness,
            2. * g.tile_width,
        ),
        (
            Edge::West,
            west + 4. * g.tile_width,
            north + 2. * g.tile_height - thickness,
            join,
            2. * g.tile_height,
        ),
        (
            Edge::East,
            west + 16. * g.tile_width,
            north + 2. * g.tile_height - thickness,
            join,
            2. * g.tile_height,
        ),
        (
            Edge::West,
            west,
            join,
            north + 36. * g.tile_height,
            2. * g.tile_height,
        ),
        (
            Edge::East,
            west + 20. * g.tile_width,
            join,
            north + 36. * g.tile_height,
            2. * g.tile_height,
        ),
        (
            Edge::South,
            north + 36. * g.tile_height,
            west - thickness,
            west + 20. * g.tile_width + thickness,
            2. * g.tile_width,
        ),
    ] {
        let mut start = from;
        while start < to - 0.001 {
            let end = (start + step).min(to);
            let mut runs = vec![(start, end)];
            if matches!(edge, Edge::South) {
                for warp in frame.boundary_warps.iter() {
                    let a = west + warp.x as f32 * 2. * g.tile_width;
                    let b = a + 2. * g.tile_width;
                    runs = runs
                        .into_iter()
                        .flat_map(|(s, e)| {
                            if a >= e || b <= s {
                                vec![(s, e)]
                            } else {
                                [(s, a.min(e)), (b.max(s), e)]
                                    .into_iter()
                                    .filter(|(a, b)| b - a > 0.001)
                                    .collect()
                            }
                        })
                        .collect();
                }
            }
            for (a, end) in runs {
                append_section(mesh, solid, textured, b, edge, at, a, end);
            }
            start = end;
        }
    }
    if mesh.authored_cells.len() == cells.len() {
        for (i, tile) in cells.iter().enumerate() {
            let x = frame.grid_origin.x + (i % g.width) as i32;
            let y = frame.grid_origin.y + (i / g.width) as i32;
            if header_wall(x, y)
                && tile.source.tile_index != 0x2d
                && mesh.authored_cells[i].is_none()
            {
                mesh.authored_cells[i] = Some("room:vermilion-stepped-boundary");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::UVec2;
    fn fixture() -> (VisualWorldFrame, GridGeometry, Vec<VisualTile>) {
        let tiles = (0..720)
            .map(|i| {
                let (x, y) = (i % 20, i / 20);
                let block = if y < 12 { HEADER[y / 4][x / 4] } else { 1 };
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    texture: Default::default(),
                    animation_frames: None,
                    priority: false,
                    source: VisualTileSource {
                        tileset_id: "game_corner".into(),
                        metatile_id: block,
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: drawing(block)[y % 4][x % 4],
                    },
                }
            })
            .collect();
        (
            VisualWorldFrame {
                map_id: "VermilionGym".into(),
                source_map_size_core_tiles: Some(UVec2::new(10, 18)),
                grid_size: UVec2::new(20, 36),
                boundary_warps: vec![IVec2::new(4, 17), IVec2::new(5, 17)].into(),
                ..Default::default()
            },
            GridGeometry {
                width: 20,
                height: 36,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            },
            tiles,
        )
    }
    fn check_outline(mesh: &TerrainMeshData, offset: Vec2) {
        let surfaces = [
            (&mesh.solid, &mesh.camera_wall_solid_ranges),
            (&mesh.textured, &mesh.camera_wall_textured_ranges),
        ];
        for (x, from, to) in [
            (32., 16., 90.),
            (128., 16., 90.),
            (0., 90., 288.),
            (160., 90., 288.),
        ] {
            let mut spans = Vec::new();
            for (surface, ranges) in surfaces {
                for (r, p) in ranges {
                    let at = if p[0] > 0. { p[2] } else { -p[2] };
                    if p[0].abs() != 1. || (at - x - offset.x).abs() > 0.001 {
                        continue;
                    }
                    let points = &surface.positions[r.clone()];
                    spans.push((
                        points
                            .iter()
                            .map(|p| p[2] - offset.y)
                            .fold(f32::INFINITY, f32::min),
                        points
                            .iter()
                            .map(|p| p[2] - offset.y)
                            .fold(f32::NEG_INFINITY, f32::max),
                    ));
                    assert!((points.iter().map(|p| p[1]).fold(0., f32::max) - 32.).abs() < 0.001);
                }
            }
            spans.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut end = from;
            assert!(!spans.is_empty(), "wall x{x}");
            for (a, b) in spans {
                assert!(a <= end + 0.001, "x{x} gap {end}..{a}");
                end = end.max(b);
            }
            assert!(end >= to - 0.001, "wall x{x} stops at {end}");
        }
        let mut north = false;
        for (surface, ranges) in surfaces {
            for (r, p) in ranges {
                let points = &surface.positions[r.clone()];
                if p[1] == 1. {
                    north = true;
                    assert!((p[2] - offset.y - 16.).abs() < 0.001);
                }
                if p[1] == -1. {
                    assert!(
                        points
                            .iter()
                            .all(|p| p[0] - offset.x <= 64.001 || p[0] - offset.x >= 95.999),
                        "both exit tiles open"
                    );
                }
                assert!(
                    !points.iter().any(|p| p[1] > 0.
                        && p[0] - offset.x > 64.001
                        && p[0] - offset.x < 95.999
                        && p[2] - offset.y > 64.001
                        && p[2] - offset.y < 95.999),
                    "central passage stays clear"
                );
            }
        }
        assert!(north, "a real rear wall must back the leader alcove");
    }
    #[test]
    fn stepped_gym_outline_connects_the_alcove_and_keeps_passage_exits_and_floor_clear() {
        let (frame, g, tiles) = fixture();
        let cells = tiles.iter().collect::<Vec<_>>();
        let mut donor = SurfaceMeshData::default();
        architecture_box(&mut donor, [0., 16., 0., 32., -3., 0.], [1.; 3]);
        let mut mesh = TerrainMeshData {
            footing_heights: vec![3.; 720],
            authored_cells: vec![None; 720],
            ..Default::default()
        };
        for y in 0..36 {
            for x in 0..20 {
                crate::mesh::append_top(&mut mesh.textured, g.bounds(x, y).into(), 0., g.uv(x, y));
            }
        }
        // Native backing can borrow paint from an unrelated floor cell. It
        // still must not create a platform in the source's black padding.
        let extra = mesh.textured.positions.len() as u32;
        crate::mesh::append_top(&mut mesh.textured, [0., 8., 16., 24.], 0., g.uv(8, 20));
        let panel_floor = mesh.textured.positions.len() as u32;
        crate::mesh::append_top(&mut mesh.textured, [0., 64., 64., 96.], 0., g.uv(8, 20));
        let original = mesh.textured.clone();
        append(
            &mut mesh,
            &frame,
            &cells,
            &g,
            None,
            &SurfaceMeshData::default(),
            &donor,
            [0., 16., -3., 0.],
        );
        check_outline(&mesh, Vec2::ZERO);
        assert_eq!(mesh.footing_heights, vec![3.; 720]);
        assert_eq!(
            mesh.authored_cells
                .iter()
                .filter(|&&p| p == Some("room:vermilion-stepped-boundary"))
                .count(),
            46
        );
        let kept = mesh
            .textured
            .indices
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        assert!(
            (extra..extra + 4).all(|i| !kept.contains(&i)),
            "borrowed floor paint cannot fill a void"
        );
        assert!(
            (panel_floor..panel_floor + 4).all(|i| !kept.contains(&i)),
            "whole illustrated-panel backing must be replaced"
        );
        for y in 0..36 {
            for x in 0..20 {
                let start = (y * 20 + x) * 4;
                let rebuilt = header_wall(x as i32, y as i32)
                    || (y < 12 && (x < 4 || x >= 16))
                    || ((8..12).contains(&y) && (x < 8 || x >= 12));
                assert_eq!(
                    kept.contains(&(start as u32)),
                    !rebuilt,
                    "old source cap/floor {x},{y}"
                );
                assert_eq!(
                    &mesh.textured.positions[start..start + 4],
                    &original.positions[start..start + 4]
                );
            }
        }
    }
    #[test]
    fn gym_outline_declines_changed_art_crops_foreign_maps_warps_and_custom_owners() {
        let (frame, g, tiles) = fixture();
        assert!(topology(
            &frame,
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            None
        ));
        for i in 0..240 {
            let mut changed = tiles.clone();
            changed[i].source.tile_index ^= 1;
            assert!(!topology(
                &frame,
                &changed.iter().collect::<Vec<_>>(),
                &g,
                None
            ));
        }
        let mut other = frame.clone();
        other.map_id = "CeladonGameCorner".into();
        assert!(!topology(
            &other,
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            None
        ));
        let mut crop = g.clone();
        crop.height = 20;
        assert!(!topology(
            &frame,
            &tiles[..400].iter().collect::<Vec<_>>(),
            &crop,
            None
        ));
        let mut other = frame.clone();
        other.boundary_warps = vec![IVec2::new(4, 0), IVec2::new(5, 17)].into();
        assert!(!topology(
            &other,
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            None
        ));
        let custom: Document = serde_json::from_value(serde_json::json!({"objects":[{
            "name":"Custom gym header","map":"VermilionGym","tileset":"game_corner","metatile":38,
            "origin":[0,0],"tiles":[[77]],"ground":1,"top_pixels":24,"depth_pixels":3}]}))
        .unwrap();
        assert!(!topology(
            &frame,
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            Some(&custom)
        ));
    }
    #[test]
    fn external_gym_uses_native_wall_template_and_preserves_support_at_shifted_origins() {
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack regression");
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
        let pack = loaded.pack();
        let map = &pack.data().maps["VermilionGym"];
        let art = &pack.runtime_files()["data/tilesets/game_corner_metatiles.bin"];
        let (frame, g, mut tiles) = fixture();
        for (i, t) in tiles.iter_mut().enumerate() {
            let (x, y) = (i % 20, i / 20);
            let block = map.blocks[y / 4 * 5 + x / 4];
            t.source.metatile_id = block;
            t.source.tile_index = art[block as usize * 16 + y % 4 * 4 + x % 4] as u16;
        }
        let cells = tiles.iter().collect::<Vec<_>>();
        assert!(topology(&frame, &cells, &g, None));
        let placements = super::super::super::resolve(&frame.map_id, &cells, &g, None);
        let dungeon = crate::mesh::modeled_dungeons::resolve(&frame.map_id, &cells, &g, None);
        for offset in [Vec2::ZERO, Vec2::new(-13., 27.)] {
            let shifted = GridGeometry {
                origin_x: offset.x,
                origin_z: offset.y,
                ..g.clone()
            };
            let mut mesh = TerrainMeshData {
                footing_heights: vec![3.; 720],
                authored_cells: vec![None; 720],
                ..Default::default()
            };
            super::super::append(
                &mut mesh,
                &frame,
                &cells,
                &shifted,
                &placements,
                &dungeon,
                None,
                None,
            );
            check_outline(&mesh, offset);
            let paint_cells = mesh
                .textured
                .indices
                .chunks_exact(3)
                .map(|tri| {
                    let uv = tri
                        .iter()
                        .map(|&i| Vec2::from_array(mesh.textured.uvs[i as usize]))
                        .sum::<Vec2>()
                        / 3.;
                    ((uv.x * 20.) as usize, (uv.y * 36.) as usize)
                })
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(
                paint_cells,
                [(0, 0), (4, 0), (8, 4), (9, 4), (8, 5), (9, 5)]
                    .into_iter()
                    .collect(),
                "only authenticated wall paint and checker-floor phases are reused"
            );
            assert_eq!(mesh.footing_heights, vec![3.; 720]);
            assert_eq!(
                mesh.authored_cells
                    .iter()
                    .filter(|&&p| p == Some("room:vermilion-stepped-boundary"))
                    .count(),
                46
            );
        }
    }
}
