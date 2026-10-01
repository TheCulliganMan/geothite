// Quiet native Gym materials. These are coplanar surface partitions, never
// modeled objects, collision changes, raised turf, or new actor footing.
fn gym_floor_map(map: &str) -> bool {
    matches!(map, "AzaleaGym" | "GoldenrodGym" | "CeladonGym")
}
fn gym_floor_style(style: InteriorFloor) -> bool {
    matches!(
        style,
        InteriorFloor::GymRose | InteriorFloor::GymGreen | InteriorFloor::GymTimber
    )
}
fn gym_floor_source(map: &str, s: &VisualTileSource) -> Option<InteriorFloor> {
    use InteriorFloor::*;
    if s.subtile_column >= 4 || s.subtile_row >= 4 {
        return None;
    }
    let phase = s.subtile_row * 4 + s.subtile_column;
    let (style, mask, tile) = match (map, s.tileset_id.as_ref()) {
        ("AzaleaGym", "elite_four_room") if s.metatile_id == 0x12 => {
            // $12 is genuine FLOOR in all four native collision quadrants.
            (
                GymGreen,
                0xffffu16,
                if 0xc140u16 & (1 << phase) != 0 {
                    0x1e
                } else {
                    0x1f
                },
            )
        }
        ("AzaleaGym", "elite_four_room") if s.metatile_id == 0x21 => {
            // The green south skirt is floor. Its two northeast WALL cells
            // remain authored source even though they share tile $1f.
            (GymGreen, 0xf800, 0x1f)
        }
        ("AzaleaGym" | "GoldenrodGym", "elite_four_room") => {
            let mask = match s.metatile_id {
                0x02 => 0xffff,
                0x03 => 0x00ff, // Keep the south warp carpet.
                0x04 => 0xcc00,
                0x06 => 0x3300,
                0x08 | 0x26 => 0xcccc,
                0x0c => 0x00cc,
                0x0e => 0x0033,
                0x25 => 0x3333,
                0x0a if map == "AzaleaGym" => 0x3333,
                0x05 if map == "GoldenrodGym" => 0xff00,
                0x0b if map == "GoldenrodGym" => 0xccff,
                0x0d if map == "GoldenrodGym" => 0x00ff,
                0x0f if map == "GoldenrodGym" => 0xffcc,
                0x11 if map == "GoldenrodGym" => 0xff33,
                0x13 if map == "GoldenrodGym" => 0x33ff,
                _ => return None,
            };
            (GymRose, mask, 0x03)
        }
        ("CeladonGym", "train_station") => match s.metatile_id {
            0x19 => (GymGreen, 0xffff, 0x57),
            0x21 => (GymGreen, 0xcccc, 0x57),
            0x22 => (GymGreen, 0x3333, 0x57),
            0x23 => (GymGreen, 0xff00, 0x57),
            0x1c => (GymTimber, 0xffff, 0x56),
            0x1d => (GymTimber, 0xcccc, 0x56),
            0x1e => (GymTimber, 0x3333, 0x56),
            0x1f => (GymTimber, 0x00ff, 0x56),
            _ => return None,
        },
        _ => return None,
    };
    (mask & (1 << phase) != 0 && s.tile_index == tile).then_some(style)
}
fn gym_floor_underlays(
    mesh: &TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    excluded: &[bool],
) -> Vec<Option<usize>> {
    if !gym_floor_map(map) {
        return Vec::new();
    }
    let mut ground = vec![None; cells.len()];
    let reserved = if excluded.is_empty() {
        vec![false; cells.len()]
    } else {
        excluded.to_vec()
    };
    // Use the kit's complete drawing and ground identity proof again. An exact
    // ownership label alone cannot authorize a stale, altered or cropped prop.
    for p in gym_scenery::resolve(map, cells, g, origin, None, &reserved) {
        let (sample, label) = p.floor_sample_and_label();
        if !p
            .indices(g.width)
            .all(|i| mesh.authored_cells.get(i) == Some(&Some(label)))
        {
            continue;
        }
        for i in p.indices(g.width) {
            ground[i] = Some(sample);
        }
    }
    ground
}
fn gym_floor_destination(
    mesh: &TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    destination: usize,
    sample: usize,
    style: InteriorFloor,
    ground: &[Option<usize>],
    excluded: &[bool],
) -> bool {
    if excluded.get(destination) == Some(&true) || excluded.get(sample) == Some(&true) {
        return false;
    }
    if ground.get(destination) == Some(&Some(sample)) {
        return true;
    }
    // Native floor must sample its own full source cell and retain its exact
    // material. A neighbor's green UV cannot turn a rose path, warp or flowerbed
    // into turf, nor may a custom-authored label gain material permission.
    destination == sample
        && mesh
            .authored_cells
            .get(destination)
            .is_none_or(Option::is_none)
        && gym_floor_source(map, &cells[destination].source) == Some(style)
}
fn gym_floor_cell(
    mesh: &mut SurfaceMeshData,
    b: [f32; 4],
    height: f32,
    style: InteriorFloor,
    world: [i32; 2],
) {
    let [x0, x1, z0, z1] = b;
    let [column, row] = world;
    let w = x1 - x0;
    let d = z1 - z0;
    match style {
        InteriorFloor::GymGreen => {
            // Quiet garden ground; no per-pixel flecks or invented grass height.
            // Broad tonal fields stay continuous when the viewport scrolls.
            let variation =
                ((column.div_euclid(8) + row.div_euclid(8)).rem_euclid(2) as f32 - 0.5) * 0.008;
            material_quad(mesh, b, height, tint(style.palette(), variation));
        }
        InteriorFloor::GymRose => {
            // Broad four-cell ceramic modules with a restrained blush/cream
            // alternation, narrow joints, and no overlapping raised overlays.
            let parity = (column.div_euclid(4) + row.div_euclid(4)).rem_euclid(2);
            let face = if parity == 0 {
                [0.70, 0.61, 0.57]
            } else {
                [0.67, 0.57, 0.54]
            };
            let joint = [0.62, 0.54, 0.51];
            let west = if column.rem_euclid(4) == 0 {
                w * 0.018
            } else {
                0.
            };
            let north = if row.rem_euclid(4) == 0 {
                d * 0.018
            } else {
                0.
            };
            material_quad(mesh, [x0, x1, z0, z0 + north], height, joint);
            material_quad(mesh, [x0, x0 + west, z0 + north, z1], height, joint);
            material_quad(mesh, [x0 + west, x1, z0 + north, z1], height, face);
        }
        InteriorFloor::GymTimber => {
            let course = row.div_euclid(2);
            let phase = course.rem_euclid(2) * 4;
            let board = (column - phase).div_euclid(8);
            let face = tint(
                style.palette(),
                ((board * 3 + course * 5).rem_euclid(5) as f32 - 2.) * 0.005,
            );
            let joint = tint(style.palette(), -0.045);
            let west = if (column - phase).rem_euclid(8) == 0 {
                w * 0.018
            } else {
                0.
            };
            let north = if row.rem_euclid(2) == 0 {
                d * 0.018
            } else {
                0.
            };
            material_quad(mesh, [x0, x1, z0, z0 + north], height, joint);
            material_quad(mesh, [x0, x0 + west, z0 + north, z1], height, joint);
            material_quad(mesh, [x0 + west, x1, z0 + north, z1], height, face);
        }
        _ => unreachable!("Gym material is selected by an exact source guard"),
    }
}

#[cfg(test)]
mod gym_floor_tests {
    use super::*;
    use std::sync::Arc;
    fn source(tileset: &str, block: u16, x: u8, y: u8, tile: u16) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from(tileset),
            metatile_id: block,
            subtile_column: x,
            subtile_row: y,
            tile_index: tile,
        }
    }
    fn tile(source: VisualTileSource, i: usize, width: usize) -> VisualTile {
        VisualTile {
            column: (i % width) as u32,
            row: (i / width) as u32,
            source,
            texture: Default::default(),
            priority: false,
        }
    }
    fn grid(width: usize, height: usize) -> GridGeometry {
        GridGeometry {
            width,
            height,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        }
    }
    fn quad(mesh: &mut TerrainMeshData, g: &GridGeometry, destination: usize, sample: usize) {
        append_top(
            &mut mesh.textured,
            g.bounds(destination % g.width, destination / g.width)
                .into(),
            2.5,
            g.uv(sample % g.width, sample / g.width),
        );
    }
    #[test]
    fn gym_floor_native_masks_reject_navigation_and_changed_phase() {
        for (map, blocks) in [
            (
                "AzaleaGym",
                vec![
                    (0x02, 16),
                    (0x03, 8),
                    (0x04, 4),
                    (0x06, 4),
                    (0x08, 8),
                    (0x0a, 8),
                    (0x0c, 4),
                    (0x0e, 4),
                    (0x25, 8),
                    (0x26, 8),
                ],
            ),
            (
                "GoldenrodGym",
                vec![
                    (0x02, 16),
                    (0x03, 8),
                    (0x04, 4),
                    (0x05, 8),
                    (0x06, 4),
                    (0x08, 8),
                    (0x0b, 12),
                    (0x0c, 4),
                    (0x0d, 8),
                    (0x0e, 4),
                    (0x0f, 12),
                    (0x11, 12),
                    (0x13, 12),
                    (0x25, 8),
                    (0x26, 8),
                ],
            ),
        ] {
            for (block, expected) in blocks {
                let mut count = 0;
                for y in 0..4 {
                    for x in 0..4 {
                        let s = source("elite_four_room", block, x, y, 0x03);
                        count += usize::from(gym_floor_source(map, &s).is_some());
                        let mut changed = s.clone();
                        changed.tile_index = 0x01;
                        assert!(gym_floor_source(map, &changed).is_none());
                        changed = s.clone();
                        changed.tileset_id = Arc::from("custom_elite_four_room");
                        assert!(gym_floor_source(map, &changed).is_none());
                        for wrong in ["VioletGym", "WillsRoom", "AzaleaGymCustom", "ViridianGym"] {
                            assert!(gym_floor_source(wrong, &s).is_none());
                        }
                    }
                }
                assert_eq!(count, expected, "{map} ${block:02x}");
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                let i = y * 4 + x;
                let t = if 0xc140u16 & (1 << i) != 0 {
                    0x1e
                } else {
                    0x1f
                };
                let mut s = source("elite_four_room", 0x12, x, y, t);
                assert_eq!(
                    gym_floor_source("AzaleaGym", &s),
                    Some(InteriorFloor::GymGreen)
                );
                s.tile_index = if t == 0x1f { 0x1e } else { 0x1f };
                assert!(gym_floor_source("AzaleaGym", &s).is_none());
                assert!(gym_floor_source("GoldenrodGym", &s).is_none());
            }
        }
        assert!(
            gym_floor_source("AzaleaGym", &source("elite_four_room", 0x21, 3, 0, 0x1f)).is_none()
        );
        assert!(
            gym_floor_source("AzaleaGym", &source("elite_four_room", 0x21, 3, 1, 0x1f)).is_none()
        );
        for (block, count, tile_id) in [
            (0x19, 16, 0x57),
            (0x21, 8, 0x57),
            (0x22, 8, 0x57),
            (0x23, 8, 0x57),
            (0x1c, 16, 0x56),
            (0x1d, 8, 0x56),
            (0x1e, 8, 0x56),
            (0x1f, 8, 0x56),
        ] {
            let accepted = (0..4)
                .flat_map(|y| (0..4).map(move |x| (x, y)))
                .filter(|&(x, y)| {
                    gym_floor_source("CeladonGym", &source("train_station", block, x, y, tile_id))
                        .is_some()
                })
                .count();
            assert_eq!(accepted, count);
        }
        for (map, ts, block, tile_id) in [
            ("GoldenrodGym", "elite_four_room", 0x07, 0x03),
            ("AzaleaGym", "elite_four_room", 0xff, 0x03),
            ("CeladonGym", "train_station", 0x20, 0x57),
            ("CeladonGym", "train_station", 0x1b, 0x57),
            ("ViridianGym", "train_station", 0x03, 0x3d),
        ] {
            for y in 0..4 {
                for x in 0..4 {
                    assert!(gym_floor_source(map, &source(ts, block, x, y, tile_id)).is_none());
                }
            }
        }
        assert!(
            gym_floor_source("AzaleaGym", &source("elite_four_room", 0x02, 4, 0, 0x03)).is_none()
        );
        assert!(
            gym_floor_source("AzaleaGym", &source("elite_four_room", 0x02, 0, 4, 0x03)).is_none()
        );
    }
    fn fixture() -> (Vec<VisualTile>, GridGeometry, TerrainMeshData) {
        let g = grid(4, 4);
        let tiles = (0..16)
            .map(|i| {
                let (x, y) = (i % 4, i / 4);
                let prop = x < 2 && y < 2;
                tile(
                    source(
                        "elite_four_room",
                        if prop { 0x0f } else { 0x02 },
                        x as u8,
                        y as u8,
                        if prop {
                            [[0x07, 0x08], [0x17, 0x18]][y][x]
                        } else {
                            0x03
                        },
                    ),
                    i,
                    4,
                )
            })
            .collect::<Vec<_>>();
        let mut m = TerrainMeshData::default();
        m.authored_cells = vec![None; 16];
        m.footing_heights = vec![3.25; 16];
        for i in 0..16 {
            let prop = i % 4 < 2 && i / 4 < 2;
            if prop {
                m.authored_cells[i] = Some("gym:leafy-display-planter");
            }
            quad(&mut m, &g, i, if prop { 2 } else { i });
        }
        (tiles, g, m)
    }
    #[test]
    fn gym_floor_model_backing_requires_complete_live_source_and_full_ownership() {
        let (tiles, g, m) = fixture();
        let refs = tiles.iter().collect::<Vec<_>>();
        let mut finished = m.clone();
        let authored = m.authored_cells.clone();
        let footing = m.footing_heights.clone();
        assert_eq!(
            finish_surfaces(&mut finished, "GoldenrodGym", &refs, &g, [0, 0]),
            16
        );
        assert!(finished.textured.indices.is_empty());
        assert_eq!(finished.authored_cells, authored);
        assert_eq!(finished.footing_heights, footing);
        assert!(finished.solid.positions.iter().all(|p| p[1] == 2.5));
        for mode in 0..5 {
            let mut changed = tiles.clone();
            let mut rejected = m.clone();
            match mode {
                0 => changed[5].source.tile_index = 0x99,
                1 => changed[5].source.subtile_column = 0,
                2 => rejected.authored_cells[5] = None,
                3 => rejected.authored_cells[5] = Some("custom:gym-planter"),
                _ => changed[5].source.metatile_id = 0x07,
            }
            assert_eq!(
                finish_surfaces(
                    &mut rejected,
                    "GoldenrodGym",
                    &changed.iter().collect::<Vec<_>>(),
                    &g,
                    [0, 0]
                ),
                12
            );
            assert_eq!(
                rejected.textured.indices.len(),
                24,
                "all four stale backing cells remain native"
            );
        }
        // Cropping the planter's bottom row cannot leave an authorized half.
        let cropped = &tiles[..4];
        let cg = grid(4, 1);
        let mut cm = TerrainMeshData::default();
        cm.authored_cells = m.authored_cells[..4].to_vec();
        for i in 0..4 {
            quad(&mut cm, &cg, i, if i < 2 { 2 } else { i });
        }
        assert_eq!(
            finish_surfaces(
                &mut cm,
                "GoldenrodGym",
                &cropped.iter().collect::<Vec<_>>(),
                &cg,
                [0, 0]
            ),
            2
        );
        assert_eq!(cm.textured.indices.len(), 12);
    }
    #[test]
    fn gym_floor_neighbor_uv_and_custom_profile_cannot_borrow_material_permission() {
        let g = grid(2, 1);
        let tiles = vec![
            tile(source("elite_four_room", 0x02, 0, 0, 0x03), 0, 2),
            tile(source("elite_four_room", 0x12, 1, 0, 0x1f), 1, 2),
        ];
        let refs = tiles.iter().collect::<Vec<_>>();
        for (destination, sample) in [(0, 1), (1, 0)] {
            let mut m = TerrainMeshData::default();
            quad(&mut m, &g, destination, sample);
            let before = m.clone();
            assert_eq!(finish_surfaces(&mut m, "AzaleaGym", &refs, &g, [0, 0]), 0);
            assert_eq!(m, before);
        }
        let mut m = TerrainMeshData::default();
        quad(&mut m, &g, 0, 0);
        m.textured.uvs[0][0] += 0.125;
        let before = m.clone();
        assert_eq!(finish_surfaces(&mut m, "AzaleaGym", &refs, &g, [0, 0]), 0);
        assert_eq!(m, before);
        let mut m = TerrainMeshData::default();
        quad(&mut m, &g, 0, 0);
        quad(&mut m, &g, 1, 1);
        assert_eq!(
            finish_surfaces_excluding(&mut m, "AzaleaGym", &refs, &g, [0, 0], &[true, false]),
            1
        );
        assert_eq!(
            m.textured.indices.len(),
            6,
            "custom-owned rose is unchanged; neighbor green finishes"
        );
        let custom = Document {
            objects: vec![crate::live_profiles::Object {
                name: "Custom floor picture".into(),
                tileset: "elite_four_room".into(),
                map: Some("AzaleaGym".into()),
                maps: None,
                metatile: 0x02,
                metatiles: None,
                origin: [0, 0],
                tiles: vec![vec![0x03]],
                ground: 0x03,
                top_pixels: 0,
                depth_pixels: 0.,
                footing_pixels: None,
                mask: Default::default(),
                parts: vec![],
            }],
            atmosphere: None,
        };
        let mut m = TerrainMeshData::default();
        quad(&mut m, &g, 0, 0);
        let before = m.clone();
        assert_eq!(
            finish_surfaces_with_profiles(&mut m, "AzaleaGym", &refs, &g, [0, 0], Some(&custom)),
            0
        );
        assert_eq!(m, before);
    }
    #[test]
    fn gym_floor_preserves_viridian_existing_stone_and_unknown_neighbors() {
        let g = grid(1, 1);
        let t = vec![tile(source("train_station", 0x03, 0, 0, 0x3d), 0, 1)];
        assert_eq!(
            interior_floor("ViridianGym", &t[0].source),
            Some(InteriorFloor::Stone)
        );
        let mut m = TerrainMeshData::default();
        quad(&mut m, &g, 0, 0);
        assert_eq!(
            finish_surfaces(
                &mut m,
                "ViridianGym",
                &t.iter().collect::<Vec<_>>(),
                &g,
                [-3, 5]
            ),
            1
        );
        let mut expected = SurfaceMeshData::default();
        floor_cell(
            &mut expected,
            [0., 8., 0., 8.],
            2.5,
            InteriorFloor::Stone,
            [-3, 5],
        );
        assert_eq!(m.solid, expected);
        let g = grid(2, 1);
        let t = vec![
            tile(source("elite_four_room", 0x02, 0, 0, 0x03), 0, 2),
            tile(source("elite_four_room", 0x03, 0, 2, 0x04), 1, 2),
        ];
        let mut m = TerrainMeshData::default();
        quad(&mut m, &g, 0, 0);
        quad(&mut m, &g, 1, 1);
        let neighbor_positions = m.textured.positions[4..].to_vec();
        let neighbor_uvs = m.textured.uvs[4..].to_vec();
        assert_eq!(
            finish_surfaces(
                &mut m,
                "AzaleaGym",
                &t.iter().collect::<Vec<_>>(),
                &g,
                [0, 0]
            ),
            1
        );
        assert_eq!(m.textured.positions, neighbor_positions);
        assert_eq!(m.textured.uvs, neighbor_uvs);
    }
    #[test]
    fn gym_floor_coplanar_partition_preserves_crop_phase_and_adjacent_geometry() {
        for style in [
            InteriorFloor::GymRose,
            InteriorFloor::GymGreen,
            InteriorFloor::GymTimber,
        ] {
            for y in -9..9 {
                for x in -9..9 {
                    let mut a = SurfaceMeshData::default();
                    let mut b = SurfaceMeshData::default();
                    gym_floor_cell(&mut a, [-4., 4., 2., 10.], 3.75, style, [x, y]);
                    gym_floor_cell(&mut b, [28., 36., -22., -14.], 3.75, style, [x, y]);
                    assert_eq!(a.colors, b.colors);
                    assert_eq!(a.indices, b.indices);
                    assert_eq!(a.normals, b.normals);
                    let mut area = 0.;
                    for q in a.positions.chunks_exact(4) {
                        assert!(q.iter().all(|p| p[1] == 3.75
                            && (-4.0..=4.).contains(&p[0])
                            && (2.0..=10.).contains(&p[2])));
                        area += (q[3][0] - q[0][0]) * (q[1][2] - q[0][2]);
                    }
                    assert!((area - 64.).abs() < 0.0001);
                    assert!(a.positions.len() <= 12);
                    for (a, b) in a.positions.iter().zip(&b.positions) {
                        assert!(near(a[0] + 32., b[0]) && near(a[2] - 24., b[2]) && a[1] == b[1]);
                    }
                }
            }
        }
        let (tiles, g, mut m) = fixture();
        material_quad(&mut m.solid, [-8., 0., 0., 8.], 12., [0.3; 3]);
        m.solid.cutaway_ranges.push(0..4);
        let before = m.clone();
        assert_eq!(
            finish_surfaces(
                &mut m,
                "GoldenrodGym",
                &tiles.iter().collect::<Vec<_>>(),
                &g,
                [-5, 7]
            ),
            16
        );
        assert_eq!(m.solid.cutaway_ranges, before.solid.cutaway_ranges);
        assert_eq!(&m.solid.positions[..4], &before.solid.positions);
        assert_eq!(&m.solid.colors[..4], &before.solid.colors);
        assert_eq!(&m.solid.indices[..6], &before.solid.indices);
        assert_eq!(m.animated_solid, before.animated_solid);
        assert_eq!(m.animated_textured, before.animated_textured);
        assert_eq!(m.footing_heights, before.footing_heights);
        assert_eq!(m.authored_cells, before.authored_cells);
    }
}
