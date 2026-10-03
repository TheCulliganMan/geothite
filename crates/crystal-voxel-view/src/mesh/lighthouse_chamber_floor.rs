// A quiet keeper-room checker. Source masks preserve the surrounding slate,
// pit, stair, cot, stool, tea table and every authored masonry boundary.
fn lighthouse_chamber_floor_source(map: &str, source: &VisualTileSource) -> bool {
    if map != "OlivineLighthouse6F"
        || source.tileset_id.as_ref() != "lighthouse"
        || source.subtile_column >= 4
        || source.subtile_row >= 4
    {
        return false;
    }
    let mask: u16 = match source.metatile_id {
        0x0b => 0xffff,
        0x06 => 0x00ff,
        0x08 => 0xccff,
        0x36 => 0x3333,
        _ => return false,
    };
    let x = source.subtile_column;
    let y = source.subtile_row;
    mask & (1 << (y * 4 + x)) != 0
        && source.tile_index == if (x + y) % 2 == 0 { 0x0d } else { 0x1d }
}
fn lighthouse_chamber_floor_destination(
    mesh: &TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    i: usize,
) -> bool {
    let s = &cells[i].source;
    if lighthouse_chamber_floor_source(map, s) {
        return true;
    }
    if map != "OlivineLighthouse6F"
        || s.tileset_id.as_ref() != "lighthouse"
        || s.subtile_column >= 4
        || s.subtile_row >= 4
    {
        return false;
    }
    // Successful whole-picture ownership is required before a floor-sampled
    // replacement can hide a source object; a nearby sample alone cannot do it.
    match mesh.authored_cells.get(i) {
        Some(Some("dungeon:lighthouse-tea-table")) => {
            (s.metatile_id == 0x06 && s.subtile_row >= 2) || s.metatile_id == 0x2d
        }
        Some(Some("dungeon:lighthouse-keeper-cot")) => {
            s.metatile_id == 0x36 && s.subtile_column >= 2
        }
        Some(Some("dungeon:lighthouse-red-stool")) => {
            s.metatile_id == 0x08 && s.subtile_column < 2 && s.subtile_row >= 2
        }
        _ => false,
    }
}
fn lighthouse_chamber_floor_edges(
    mesh: &TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    i: usize,
) -> [bool; 4] {
    let x = i % g.width;
    let y = i / g.width;
    // An offscreen edge is unknown, not an invented room perimeter.
    [
        y.checked_sub(1).map(|y| y * g.width + x),
        (x + 1 < g.width).then_some(i + 1),
        (y + 1 < g.height).then_some(i + g.width),
        x.checked_sub(1).map(|x| y * g.width + x),
    ]
    .map(|neighbor| {
        neighbor.is_some_and(|j| !lighthouse_chamber_floor_destination(mesh, map, cells, j))
    })
}
fn lighthouse_chamber_floor_cell(
    mesh: &mut SurfaceMeshData,
    b: [f32; 4],
    height: f32,
    world: [i32; 2],
    edges: [bool; 4],
) {
    let [x0, x1, z0, z1] = b;
    let [column, row] = world;
    let w = x1 - x0;
    let d = z1 - z0;
    // Broad 2x2-source-cell checks with restrained luminance contrast. These
    // coplanar partitions preserve exact area and original support elevation.
    let parity = (column.div_euclid(2) + row.div_euclid(2)).rem_euclid(2);
    let color = if parity == 0 {
        [0.54, 0.56, 0.51]
    } else {
        [0.48, 0.52, 0.48]
    };
    let grout = [0.435, 0.465, 0.43];
    let trim = [0.56, 0.53, 0.43];
    let west = if edges[3] {
        w * 0.11
    } else if column.rem_euclid(2) == 0 {
        w * 0.016
    } else {
        0.
    };
    let north = if edges[0] {
        d * 0.11
    } else if row.rem_euclid(2) == 0 {
        d * 0.016
    } else {
        0.
    };
    let east = if edges[1] { w * 0.11 } else { 0. };
    let south = if edges[2] { d * 0.11 } else { 0. };
    material_quad(
        mesh,
        [x0, x1, z0, z0 + north],
        height,
        if edges[0] { trim } else { grout },
    );
    material_quad(mesh, [x0, x1, z1 - south, z1], height, trim);
    material_quad(
        mesh,
        [x0, x0 + west, z0 + north, z1 - south],
        height,
        if edges[3] { trim } else { grout },
    );
    material_quad(mesh, [x1 - east, x1, z0 + north, z1 - south], height, trim);
    material_quad(
        mesh,
        [x0 + west, x1 - east, z0 + north, z1 - south],
        height,
        color,
    );
}
#[cfg(test)]
mod lighthouse_chamber_floor_tests {
    use super::*;
    use std::sync::Arc;
    const MAP: &str = "OlivineLighthouse6F";
    fn source(block: u16, x: u8, y: u8) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from("lighthouse"),
            metatile_id: block,
            subtile_column: x,
            subtile_row: y,
            tile_index: if (x + y) % 2 == 0 { 0x0d } else { 0x1d },
        }
    }
    #[test]
    fn lighthouse_chamber_floor_masks_and_negative_controls() {
        for (block, count) in [(0x0b, 16), (0x06, 8), (0x08, 12), (0x36, 8)] {
            let mut matched = 0;
            for y in 0..4 {
                for x in 0..4 {
                    let s = source(block, x, y);
                    matched += usize::from(lighthouse_chamber_floor_source(MAP, &s));
                    let mut changed = s.clone();
                    changed.tile_index ^= 0x10;
                    assert!(!lighthouse_chamber_floor_source(MAP, &changed));
                    changed = s.clone();
                    changed.tileset_id = Arc::from("custom_lighthouse");
                    assert!(!lighthouse_chamber_floor_source(MAP, &changed));
                    for map in [
                        "OlivineLighthouse5F",
                        "FastShip1F",
                        "OlivineLighthouse6FBeta",
                    ] {
                        assert!(!lighthouse_chamber_floor_source(map, &s));
                    }
                }
            }
            assert_eq!(matched, count);
        }
        for b in [
            0x27, 0x28, 0x29, 0x2d, 0x2e, 0x31, 0x3a, 0x3c, 0x3d, 0x3e, 0xff,
        ] {
            for y in 0..4 {
                for x in 0..4 {
                    assert!(!lighthouse_chamber_floor_source(MAP, &source(b, x, y)));
                }
            }
        }
        assert!(!lighthouse_chamber_floor_source(MAP, &source(0x0b, 4, 0)));
        assert!(!lighthouse_chamber_floor_source(MAP, &source(0x0b, 0, 4)));
    }
    fn tiles() -> Vec<VisualTile> {
        [source(0x0b, 0, 0), source(0x36, 2, 0)]
            .into_iter()
            .enumerate()
            .map(|(i, source)| VisualTile {
                animation_frames: None,
                column: i as u32,
                row: 0,
                texture: Default::default(),
                priority: false,
                source,
            })
            .collect()
    }
    fn grid() -> GridGeometry {
        GridGeometry {
            width: 2,
            height: 1,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        }
    }
    fn mesh(sample: usize) -> TerrainMeshData {
        let g = grid();
        let mut m = TerrainMeshData::default();
        append_top(&mut m.textured, [8., 16., 0., 8.], 2.5, g.uv(sample, 0));
        m.authored_cells = vec![None, None];
        m.footing_heights = vec![1.25, 2.5];
        m
    }
    #[test]
    fn lighthouse_chamber_floor_requires_real_sample_and_successful_model() {
        let t = tiles();
        let refs: Vec<_> = t.iter().collect();
        let g = grid();
        let mut m = mesh(0);
        let before = m.clone();
        assert_eq!(finish_surfaces(&mut m, MAP, &refs, &g, [0, 0]), 0);
        assert_eq!(m, before);
        m.authored_cells[1] = Some("dungeon:lighthouse-keeper-cot");
        let footing = m.footing_heights.clone();
        let authored = m.authored_cells.clone();
        assert_eq!(finish_surfaces(&mut m, MAP, &refs, &g, [0, 0]), 1);
        assert_eq!(m.footing_heights, footing);
        assert_eq!(m.authored_cells, authored);
        assert!(m.solid.positions.iter().all(|v| v[1] == 2.5));
        assert!(m.solid.cutaway_ranges.is_empty());
        for (sample, label) in [
            (1, "dungeon:lighthouse-keeper-cot"),
            (0, "dungeon:lighthouse-masonry"),
            (0, "interior/bed_green"),
        ] {
            let mut m = mesh(sample);
            m.authored_cells[1] = Some(label);
            let before = m.clone();
            assert_eq!(finish_surfaces(&mut m, MAP, &refs, &g, [0, 0]), 0);
            assert_eq!(m, before);
        }
        let mut m = mesh(0);
        m.authored_cells[1] = Some("dungeon:lighthouse-keeper-cot");
        m.textured.uvs[0][0] += 0.1;
        let before = m.clone();
        assert_eq!(finish_surfaces(&mut m, MAP, &refs, &g, [0, 0]), 0);
        assert_eq!(m, before);
    }
    #[test]
    fn lighthouse_chamber_checker_partitions_coplanar_area_and_preserves_crop_phase() {
        for x in -5..5 {
            for y in -5..5 {
                for mask in 0..16 {
                    let mut s = SurfaceMeshData::default();
                    lighthouse_chamber_floor_cell(
                        &mut s,
                        [-2., 6., 5., 13.],
                        3.75,
                        [x, y],
                        std::array::from_fn(|i| mask & (1 << i) != 0),
                    );
                    assert!(s.positions.iter().all(|v| v[1] == 3.75
                        && (-2.0..=6.).contains(&v[0])
                        && (5.0..=13.).contains(&v[2])));
                    let area: f32 = s
                        .indices
                        .chunks_exact(3)
                        .map(|tri| {
                            let a = Vec3::from_array(s.positions[tri[0] as usize]);
                            let b = Vec3::from_array(s.positions[tri[1] as usize]);
                            let c = Vec3::from_array(s.positions[tri[2] as usize]);
                            let cross = (b - a).cross(c - a);
                            assert!(cross.y > 0.);
                            cross.length() * 0.5
                        })
                        .sum();
                    assert!((area - 64.).abs() < 0.001);
                    let mut shifted = SurfaceMeshData::default();
                    lighthouse_chamber_floor_cell(
                        &mut shifted,
                        [-2., 6., 5., 13.],
                        3.75,
                        [x + 4, y - 4],
                        std::array::from_fn(|i| mask & (1 << i) != 0),
                    );
                    assert_eq!(s, shifted);
                }
            }
        }
        let g = grid();
        let t = tiles();
        let refs: Vec<_> = t.iter().collect();
        let m = mesh(0);
        assert_eq!(
            lighthouse_chamber_floor_edges(&m, MAP, &refs, &g, 0),
            [false, true, false, false]
        );
    }
}
