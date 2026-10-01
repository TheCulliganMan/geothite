// Restrained ship finishes keep native floor support and every source opening.
mod ship_floor_source {
    include!("ship_floor_source.rs");
}
use ship_floor_source::Style as ShipFloor;
fn ship_floor_map(map: &str) -> bool {
    ship_floor_source::applies(map)
}
fn ship_floor_source(map: &str, s: &VisualTileSource) -> Option<ShipFloor> {
    ship_floor_source::style(
        map,
        s.tileset_id.as_ref(),
        s.metatile_id,
        s.subtile_column,
        s.subtile_row,
        s.tile_index,
    )
}
fn ship_floor_underlays(
    mesh: &TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    excluded: &[bool],
) -> Vec<Option<usize>> {
    if !ship_floor_map(map) {
        return Vec::new();
    }
    let mut ground = vec![None; cells.len()];
    let reserved = if excluded.is_empty() {
        vec![false; cells.len()]
    } else {
        excluded.to_vec()
    };
    // Resolve whole current drawings again. A label on a stale, clipped or
    // changed drawing does not authorize covering it with opaque floor.
    for p in ship_rooms::resolve(map, cells, g, None, &reserved) {
        let valid = p.indices(g.width).all(|i| {
            let (sample, label) = p.floor_sample_and_label(&cells[i].source);
            !reserved[i]
                && !reserved[sample]
                && mesh.authored_cells.get(i) == Some(&Some(label))
                && matches!(
                    ship_floor_source(map, &cells[sample].source),
                    Some(ShipFloor::Cabin | ShipFloor::Mess)
                )
        });
        if valid {
            for i in p.indices(g.width) {
                ground[i] = Some(p.floor_sample_and_label(&cells[i].source).0);
            }
        }
    }
    // Reuse the existing donor and original supporting plane for the rack,
    // barrel and partial berth. This does not append or move those models.
    for p in modeled_dungeons::resolve(map, cells, g, None) {
        let (sample, label) = p.floor_sample_and_label();
        if reserved[sample]
            || !matches!(
                ship_floor_source(map, &cells[sample].source),
                Some(ShipFloor::Cabin | ShipFloor::Mess)
            )
            || !p.indices(g.width).all(|i| {
                let s = &cells[i].source;
                !reserved[i]
                    && mesh.authored_cells.get(i) == Some(&Some(label))
                    && ship_floor_source::fixture_source(
                        map,
                        label,
                        s.tileset_id.as_ref(),
                        s.metatile_id,
                        s.subtile_column,
                        s.subtile_row,
                        s.tile_index,
                    )
            })
        {
            continue;
        }
        for i in p.indices(g.width) {
            ground[i] = Some(sample);
        }
    }
    ground
}
fn ship_floor_destination(
    mesh: &TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    destination: usize,
    sample: usize,
    style: ShipFloor,
    ground: &[Option<usize>],
    excluded: &[bool],
) -> bool {
    if excluded.get(destination) == Some(&true) || excluded.get(sample) == Some(&true) {
        return false;
    }
    if ground.get(destination) == Some(&Some(sample)) {
        return matches!(style, ShipFloor::Cabin | ShipFloor::Mess);
    }
    destination == sample
        && mesh
            .authored_cells
            .get(destination)
            .is_none_or(Option::is_none)
        && ship_floor_source(map, &cells[destination].source) == Some(style)
}
fn ship_floor_cell(
    mesh: &mut SurfaceMeshData,
    b: [f32; 4],
    height: f32,
    style: ShipFloor,
    world: [i32; 2],
) {
    let [x0, x1, z0, z1] = b;
    let [column, row] = world;
    let w = x1 - x0;
    let d = z1 - z0;
    let base = style.palette();
    match style {
        ShipFloor::Cabin => {
            // Broad muted wool checks, with a barely visible woven course.
            // Global phase means identical carpet across every viewport crop.
            let check = (column.div_euclid(4) + row.div_euclid(4)).rem_euclid(2);
            let face = tint(base, if check == 0 { 0.012 } else { -0.012 });
            let west = if column.rem_euclid(4) == 0 {
                w * 0.016
            } else {
                0.
            };
            material_quad(mesh, [x0, x0 + west, z0, z1], height, tint(face, -0.015));
            for line in 0..3 {
                let a = z0 + d * line as f32 / 3.;
                let b = z0 + d * (line + 1) as f32 / 3.;
                material_quad(
                    mesh,
                    [x0 + west, x1, a, b],
                    height,
                    tint(face, if line == 1 { -0.003 } else { 0.003 }),
                );
            }
        }
        ShipFloor::Mess => {
            // Large low-contrast linoleum squares distinguish the mess from
            // soft cabin carpet while sharing the bulkheads' warm ivory.
            let check = (column.div_euclid(2) + row.div_euclid(2)).rem_euclid(2);
            let face = tint(base, if check == 0 { 0.016 } else { -0.016 });
            let west = if column.rem_euclid(2) == 0 {
                w * 0.015
            } else {
                0.
            };
            let north = if row.rem_euclid(2) == 0 {
                d * 0.015
            } else {
                0.
            };
            let joint = tint(base, -0.035);
            material_quad(mesh, [x0, x1, z0, z0 + north], height, joint);
            material_quad(mesh, [x0, x0 + west, z0 + north, z1], height, joint);
            material_quad(mesh, [x0 + west, x1, z0 + north, z1], height, face);
        }
        ShipFloor::Deck => {
            // Broad nonslip panels, sparse low-relief visual tread, no raised
            // strips or bump in actor footing. Five quads maximum per cell.
            let panel = column.div_euclid(4);
            let course = row.div_euclid(2);
            let face = tint(
                base,
                ((panel * 3 + course).rem_euclid(3) as f32 - 1.) * 0.008,
            );
            let west = if column.rem_euclid(4) == 0 {
                w * 0.035
            } else {
                0.
            };
            let north = if row.rem_euclid(2) == 0 {
                d * 0.030
            } else {
                0.
            };
            let joint = tint(base, -0.050);
            material_quad(mesh, [x0, x1, z0, z0 + north], height, joint);
            material_quad(mesh, [x0, x0 + west, z0 + north, z1], height, joint);
            let tread_start = z0 + d * 0.57;
            let tread_end = z0 + d * 0.60;
            material_quad(mesh, [x0 + west, x1, z0 + north, tread_start], height, face);
            material_quad(
                mesh,
                [x0 + west, x1, tread_start, tread_end],
                height,
                tint(face, 0.018),
            );
            material_quad(mesh, [x0 + west, x1, tread_end, z1], height, face);
        }
        ShipFloor::Border => {
            // Retain the separate native edge course as navy nonslip edging
            // with a fine brass insert. Never infer a border at a crop edge.
            let west = if column.rem_euclid(4) == 0 {
                w * 0.035
            } else {
                0.
            };
            let north = z0 + d * 0.08;
            let inset = z1 - d * 0.065;
            material_quad(mesh, [x0, x1, z0, north], height, tint(base, -0.045));
            material_quad(
                mesh,
                [x0, x0 + west, north, inset],
                height,
                tint(base, -0.045),
            );
            material_quad(mesh, [x0 + west, x1, north, inset], height, base);
            material_quad(mesh, [x0, x1, inset, z1], height, [0.52, 0.49, 0.37]);
        }
    }
}

#[cfg(test)]
mod ship_floor_tests {
    use super::*;
    use std::sync::Arc;
    fn source(block: u16, x: u8, y: u8, tile: u16) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from("lighthouse"),
            metatile_id: block,
            subtile_column: x,
            subtile_row: y,
            tile_index: tile,
        }
    }
    fn fixture() -> (Vec<VisualTile>, GridGeometry, TerrainMeshData) {
        let g = GridGeometry {
            width: 2,
            height: 1,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let tiles = [source(0x0b, 0, 0, 0x0d), source(0x0b, 1, 0, 0x1d)]
            .into_iter()
            .enumerate()
            .map(|(i, source)| VisualTile {
                column: i as _,
                row: 0,
                texture: Handle::default(),
                priority: false,
                source,
            })
            .collect();
        let mut mesh = TerrainMeshData::default();
        append_top(&mut mesh.textured, [8., 16., 0., 8.], 3.75, g.uv(1, 0));
        mesh.footing_heights = vec![0., 3.75];
        mesh.authored_cells = vec![None, None];
        (tiles, g, mesh)
    }
    #[test]
    fn ship_floor_own_sample_preserves_support_and_rejects_custom_unknown_and_partial_quads() {
        let map = "FastShipB1F";
        let (tiles, g, mesh) = fixture();
        let refs: Vec<_> = tiles.iter().collect();
        let mut finished = mesh.clone();
        assert_eq!(finish_surfaces(&mut finished, map, &refs, &g, [-3, 5]), 1);
        assert!(finished.textured.indices.is_empty());
        assert!(finished.solid.positions.iter().all(|p| p[1] == 3.75));
        assert_eq!(finished.footing_heights, mesh.footing_heights);
        assert_eq!(finished.authored_cells, mesh.authored_cells);
        for mode in 0..6 {
            let mut changed = mesh.clone();
            let mut cells = tiles.clone();
            match mode {
                0 => changed.authored_cells[1] = Some("custom/ship-floor"),
                1 => cells[1].source.metatile_id = 0xff,
                2 => cells[1].source.tile_index ^= 1,
                3 => cells[1].source.tileset_id = Arc::from("custom-lighthouse"),
                4 => {
                    let (u0, u1, v0, v1) = g.uv(0, 0);
                    changed.textured.uvs = vec![[u0, v0], [u0, v1], [u1, v1], [u1, v0]];
                }
                _ => changed.textured.uvs[0][0] += 0.01,
            }
            let before = changed.clone();
            let refs: Vec<_> = cells.iter().collect();
            assert_eq!(
                finish_surfaces(&mut changed, map, &refs, &g, [0, 0]),
                0,
                "mode {mode}"
            );
            assert_eq!(changed, before);
        }
        let mut excluded = mesh.clone();
        assert_eq!(
            finish_surfaces_excluding(&mut excluded, map, &refs, &g, [0, 0], &[false, true]),
            0
        );
        assert_eq!(excluded, mesh);
    }
    #[test]
    fn ship_floor_live_profile_has_first_refusal_even_for_exact_floor_art() {
        let (tiles, g, mut mesh) = fixture();
        let refs: Vec<_> = tiles.iter().collect();
        let custom = Document {
            objects: vec![crate::live_profiles::Object {
                name: "Custom ship floor picture".into(),
                tileset: "lighthouse".into(),
                map: Some("FastShipB1F".into()),
                maps: None,
                metatile: 0x0b,
                metatiles: None,
                origin: [1, 0],
                tiles: vec![vec![0x1d]],
                ground: 0x0d,
                top_pixels: 0,
                depth_pixels: 0.,
                footing_pixels: None,
                mask: Default::default(),
                parts: vec![],
            }],
            atmosphere: None,
        };
        let before = mesh.clone();
        assert_eq!(
            finish_surfaces_with_profiles(
                &mut mesh,
                "FastShipB1F",
                &refs,
                &g,
                [0, 0],
                Some(&custom)
            ),
            0
        );
        assert_eq!(mesh, before);
    }
    #[test]
    fn ship_floor_backing_requires_complete_model_ownership_and_the_exact_existing_donor() {
        let map = "FastShipCabins_NNW_NNE_NE";
        let g = GridGeometry {
            width: 8,
            height: 8,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut tiles: Vec<_> = (0..64)
            .map(|i| VisualTile {
                column: (i % 8) as _,
                row: (i / 8) as _,
                texture: Handle::default(),
                priority: false,
                source: source(
                    0x0b,
                    (i % 8 % 4) as _,
                    (i / 8 % 4) as _,
                    if (i % 8 + i / 8) % 2 == 0 { 0x0d } else { 0x1d },
                ),
            })
            .collect();
        let rows = [
            [0x09, 0x0a, 0x0a, 0x0c],
            [0x19, 0x1a, 0x2c, 0x1c],
            [0x14, 0x82, 0x82, 0x35],
            [0x0b, 0x80, 0x81, 0x0b],
        ];
        for y in 0..4 {
            for x in 0..4 {
                tiles[(y + 2) * 8 + x + 4].source = source(
                    if y < 2 { 0x06 } else { 0x2c },
                    x as _,
                    ((y + 2) % 4) as _,
                    rows[y][x],
                );
            }
        }
        let refs: Vec<_> = tiles.iter().collect();
        let placements = ship_rooms::resolve(map, &refs, &g, None, &vec![false; 64]);
        assert_eq!(placements.len(), 1);
        let p = &placements[0];
        let mut mesh = TerrainMeshData::default();
        mesh.authored_cells = vec![None; 64];
        mesh.footing_heights = vec![0.; 64];
        assert!(ship_rooms::append(
            &mut mesh,
            &refs,
            &g,
            p,
            &mut vec![false; 64]
        ));
        let original = mesh.clone();
        assert_eq!(finish_surfaces(&mut mesh, map, &refs, &g, [0, 0]), 16);
        assert_eq!(mesh.footing_heights, original.footing_heights);
        assert_eq!(mesh.authored_cells, original.authored_cells);
        let n = original.solid.positions.len();
        let k = original.solid.indices.len();
        assert_eq!(&mesh.solid.positions[..n], &original.solid.positions);
        assert_eq!(&mesh.solid.colors[..n], &original.solid.colors);
        assert_eq!(&mesh.solid.normals[..n], &original.solid.normals);
        assert_eq!(&mesh.solid.uvs[..n], &original.solid.uvs);
        assert_eq!(&mesh.solid.indices[..k], &original.solid.indices);
        assert_eq!(mesh.solid.cutaway_ranges, original.solid.cutaway_ranges);
        // An owner label is insufficient if one source pixel, owner, or donor
        // is changed. No incomplete model footprint gains any floor repaint.
        for mode in 0..4 {
            let mut changed = original.clone();
            let mut cells = tiles.clone();
            let mut excluded = vec![false; 64];
            let i = 2 * 8 + 4;
            match mode {
                0 => cells[i].source.tile_index ^= 1,
                1 => changed.authored_cells[i] = None,
                2 => excluded[i] = true,
                _ => excluded[p.floor_sample_and_label(&cells[i].source).0] = true,
            }
            let before = changed.clone();
            let refs: Vec<_> = cells.iter().collect();
            assert_eq!(
                finish_surfaces_excluding(&mut changed, map, &refs, &g, [0, 0], &excluded),
                0,
                "mode {mode}"
            );
            assert_eq!(changed, before);
        }
    }
    #[test]
    fn ship_floor_partitions_are_coplanar_bounded_and_stable_across_negative_crops() {
        for style in [
            ShipFloor::Cabin,
            ShipFloor::Mess,
            ShipFloor::Deck,
            ShipFloor::Border,
        ] {
            for y in -5..6 {
                for x in -7..8 {
                    let mut a = SurfaceMeshData::default();
                    let mut b = SurfaceMeshData::default();
                    ship_floor_cell(&mut a, [-4., 4., 2., 10.], 2.75, style, [x, y]);
                    ship_floor_cell(&mut b, [28., 36., -22., -14.], 2.75, style, [x, y]);
                    assert_eq!(a.colors, b.colors);
                    assert_eq!(a.indices, b.indices);
                    assert_eq!(a.normals, b.normals);
                    assert!(a.indices.len() <= 30); // At most ten triangles per source cell.
                    let mut area = 0.;
                    for q in a.positions.chunks_exact(4) {
                        assert!(q.iter().all(|p| p[1] == 2.75
                            && (-4. ..=4.).contains(&p[0])
                            && (2. ..=10.).contains(&p[2])));
                        area += (q[3][0] - q[0][0]) * (q[1][2] - q[0][2]);
                    }
                    assert!(near(area, 64.));
                    for (a, b) in a.positions.iter().zip(&b.positions) {
                        assert!(near(a[0] + 32., b[0]) && a[1] == b[1] && near(a[2] - 24., b[2]));
                    }
                }
            }
        }
    }
}
