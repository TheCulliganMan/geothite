// Whole-source Game Corner architecture. Rectangles describe physical pieces,
// leaving the checker floor, entrance and service access in the native layout.
fn casino_drawing(block: u16) -> Option<[u16; 16]> {
    Some(match block {
        0x02 | 0x0d => [
            2, 2, 2, 2, 2, 2, 2, 2, 1, 0x10, 1, 0x10, 0x11, 0x12, 0x11, 0x12,
        ],
        0x04 => [
            0x16, 0x17, 0x17, 0x18, 0x26, 0x27, 0x27, 0x28, 0x36, 0x37, 0x37, 0x38, 0x11, 0x12,
            0x11, 0x12,
        ],
        0x09 => [
            1, 0x10, 1, 0x10, 0x11, 0x12, 0x11, 0x12, 0x0e, 0x0e, 0x0e, 0x0e, 0x1e, 0x1e, 0x1e,
            0x1e,
        ],
        0x10 => [
            0x4d, 0x4d, 0x4d, 0x4d, 0xc4, 0x51, 0x4d, 0x4d, 0x0e, 0x0e, 0x0e, 0x0e, 0x1e, 0x1e,
            0x1e, 0x1e,
        ],
        0x11 => [
            0x87, 0x88, 0x4d, 0x4d, 0x97, 0x98, 0x4d, 0x4d, 0x0e, 0x0e, 0x0e, 0x0e, 0x1e, 0x1e,
            0x1e, 0x1e,
        ],
        0x13 => [
            2, 2, 2, 2, 4, 5, 2, 2, 0x14, 0x15, 1, 0x10, 0x14, 0x15, 0x11, 0x12,
        ],
        0x14 => [
            0x14, 0x15, 1, 0x10, 0x14, 0x15, 0x11, 0x12, 0x14, 0x84, 0x0e, 0x0e, 0x3d, 0x1e, 0x1e,
            0x1e,
        ],
        0x1b => [
            1, 0x10, 0x85, 0x86, 0x11, 0x12, 0x85, 0x86, 0x0e, 0x0e, 0x94, 0x86, 0x1e, 0x1e, 0x1e,
            0x1f,
        ],
        0x1c => [
            2, 2, 2, 2, 2, 2, 0x95, 0x96, 1, 0x10, 0x85, 0x86, 0x11, 0x12, 0x85, 0x86,
        ],
        0x27 => [
            1, 0x10, 1, 0x10, 0x34, 0x35, 0x11, 0x12, 0x40, 0x41, 0x0e, 0x0e, 0x1e, 0x1e, 0x1e,
            0x1e,
        ],
        0x28 => [
            2, 2, 6, 7, 2, 2, 8, 9, 1, 0x10, 1, 0x10, 0x11, 0x12, 0x11, 0x12,
        ],
        0x2b => [
            2, 2, 2, 2, 2, 2, 0xa4, 0xa5, 1, 0x10, 0xb4, 0xb5, 0x11, 0x12, 0x32, 0x33,
        ],
        0x2f => [
            2, 2, 2, 2, 2, 2, 0x89, 0x8a, 0x0a, 0x0b, 0x8c, 0x8d, 0x1a, 0x1b, 0x99, 0x9a,
        ],
        0x30 => [
            2, 2, 2, 2, 0x8a, 0x8b, 2, 2, 0x8d, 0x8e, 0x0a, 0x0b, 0x9b, 0x9c, 0x1a, 0x1b,
        ],
        _ => return None,
    })
}

fn append_casino_candidates(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    use ModelKind::*;
    if !crate::casino::is_game_corner_map(map) {
        return;
    }
    let Some(ground) = ground_sample(cells, map, "game_corner", 1) else {
        return;
    };
    for (index, tile) in cells.iter().enumerate() {
        let s = &tile.source;
        if s.tileset_id.as_ref() != "game_corner" || s.subtile_column != 0 || s.subtile_row != 0 {
            continue;
        }
        // The Prize Room's complete booth owns these three blocks together.
        // If that course is changed or reserved, do not turn the remaining
        // individual blocks into unrelated house windows and counters.
        if map == "CeladonGameCornerPrizeRoom" && matches!(s.metatile_id, 0x10 | 0x11) {
            continue;
        }
        let Some(drawing) = casino_drawing(s.metatile_id) else {
            continue;
        };
        let (column, row) = (index % g.width, index / g.width);
        if column + 4 > g.width || row + 4 > g.height {
            continue;
        }
        if !(0..4).all(|y| {
            (0..4).all(|x| {
                let t = &cells[(row + y) * g.width + column + x].source;
                t.tileset_id == s.tileset_id
                    && t.metatile_id == s.metatile_id
                    && t.subtile_column as usize == x
                    && t.subtile_row as usize == y
                    && t.tile_index == drawing[y * 4 + x]
            })
        }) {
            continue;
        }
        // This table crosses two native blocks. A lone half never becomes
        // a counter: require both complete drawings before owning the table.
        let whole_game_table = map == "GoldenrodGameCorner" && s.metatile_id == 0x2f
            && column + 8 <= g.width && (0..4).all(|y| (0..4).all(|x| {
                let other = &cells[(row + y) * g.width + column + 4 + x].source;
                other.tileset_id.as_ref() == "game_corner" && other.metatile_id == 0x30
                    && other.subtile_column as usize == x && other.subtile_row as usize == y
                    && other.tile_index == casino_drawing(0x30).unwrap()[y * 4 + x]
            }));
        let mut piece = |kind, rect: [usize; 4], depth, rise, base| {
            let [x, y, w, h] = rect;
            let p = Placement {
                kind,
                column: column + x,
                row: row + y,
                width: w,
                height: h,
                ground,
                depth_pixels: depth,
                height_pixels: rise,
                base_pixels: base,
                footing_pixels: None,
            };
            if p.indices(g.width).any(|i| claimed[i]) {
                return;
            }
            for i in p.indices(g.width) {
                claimed[i] = true;
            }
            out.push(p);
        };
        match s.metatile_id {
            0x02 | 0x0d => piece(ArcadeWall, [0, 0, 4, 2], 3.0, 28.0, 0.0),
            0x04 => piece(ArcadePrizeRack, [0, 0, 4, 3], 10.0, 28.0, 0.0),
            0x09 => piece(ReceptionCounter, [0, 2, 4, 2], 16.0, 8.0, 0.0),
            0x10 | 0x11 => {
                piece(WindowWall, [0, 0, 4, 2], 3.0, 28.0, 0.0);
                piece(ReceptionCounter, [0, 2, 4, 2], 16.0, 8.0, 0.0);
            }
            0x13 => {
                piece(ArcadeWall, [0, 0, 2, 1], 3.0, 28.0, 0.0);
                piece(ArcadeWall, [2, 0, 2, 2], 3.0, 28.0, 0.0);
                piece(ReceptionCounter, [0, 1, 2, 3], 24.0, 8.0, 0.0);
            }
            0x14 | 0x1b => {
                piece(
                    ReceptionCounter,
                    [if s.metatile_id == 0x14 { 0 } else { 2 }, 0, 2, 2],
                    16.0,
                    8.0,
                    0.0,
                );
                piece(ReceptionCounter, [0, 2, 4, 2], 16.0, 8.0, 0.0);
            }
            0x1c => {
                piece(ArcadeWall, [0, 0, 4, 2], 3.0, 28.0, 0.0);
                piece(ReceptionCounter, [2, 2, 2, 2], 16.0, 8.0, 0.0);
            }
            0x27 => {
                piece(LabWorkstation, [0, 1, 2, 1], 8.0, 20.0, 0.0);
                piece(ReceptionCounter, [0, 2, 4, 2], 16.0, 8.0, 0.0);
            }
            0x28 => {
                piece(ArcadeWall, [0, 0, 2, 2], 3.0, 28.0, 0.0);
                piece(PictureFrame, [2, 0, 2, 2], 2.0, 14.0, 12.0);
            }
            0x2b => {
                piece(ArcadeWall, [0, 0, 2, 2], 3.0, 28.0, 0.0);
                piece(ArcadeWall, [2, 0, 2, 1], 3.0, 28.0, 0.0);
                piece(LabWorkstation, [2, 1, 2, 3], 12.0, 22.0, 0.0);
            }
            0x2f | 0x30 => {
                let table_x = if s.metatile_id == 0x2f { 2 } else { 0 };
                piece(ArcadeWall, [0, 0, 4, 1], 3.0, 28.0, 0.0);
                piece(ArcadeWall, [2 - table_x, 1, 2, 1], 3.0, 28.0, 0.0);
                if whole_game_table {
                    piece(DiningTable, [2, 1, 4, 3], 24.0, 10.0, 0.0);
                }
                piece(ArcadeStool, [2 - table_x, 2, 2, 2], 12.0, 7.0, 0.0);
            }
            _ => unreachable!(),
        }
    }
}


fn append_arcade_game_table(mesh: &mut SurfaceMeshData, g: &GridGeometry,
    p: &Placement, bounds: [f32; 4]) {
    let [w,e,n,s] = bounds;
    let scale = g.tile_height / 8.;
    let sample = |pixel: [usize; 2]| {
        let uv = g.uv(p.column + pixel[0] / 8, p.row + pixel[1] / 8);
        [uv.0 + (uv.1 - uv.0) * ((pixel[0] % 8) as f32 + 0.5) / 8.,
         uv.2 + (uv.3 - uv.2) * ((pixel[1] % 8) as f32 + 0.5) / 8.]
    };
    let mut box_part = |b, pixel| {
        let first = mesh.positions.len();
        architecture_box(mesh, b, [1.; 3]);
        mesh.uvs[first..].fill(sample(pixel));
    };
    box_part([w,e,8.5*scale,9.5*scale,n,s], [1,16]);
    for x in [w+1.5*scale,e-3.9*scale] {
        for z in [n+1.5*scale,s-3.9*scale] {
            box_part([x,x+2.4*scale,0.,8.5*scale,z,z+2.4*scale], [0,17]);
        }
    }
    box_part([w+scale,e-scale,9.5*scale,10.*scale,n+scale,s-scale], [8,4]);
    // Four closed gray rim rails surround one continuous blue playing board.
    for b in [[w,e,9.5*scale,10.*scale,n,n+scale],
        [w,e,9.5*scale,10.*scale,s-scale,s],
        [w,w+scale,9.5*scale,10.*scale,n+scale,s-scale],
        [e-scale,e,9.5*scale,10.*scale,n+scale,s-scale]] {
        box_part(b, [1,16]);
    }
    drop(box_part);
    // The native board owns its two card markers and blue cloth. Its front
    // leg row stays on the actual gray supports rather than on the table top.
    for row in 0..2 { for column in 0..4 {
        let x0=w+scale+(e-w-2.*scale)*column as f32/4.;
        let x1=w+scale+(e-w-2.*scale)*(column+1) as f32/4.;
        let z0=n+scale+(s-n-2.*scale)*row as f32/2.;
        let z1=n+scale+(s-n-2.*scale)*(row+1) as f32/2.;
        append_top(mesh,[x0,x1,z0,z1],10.001*scale,g.uv(p.column+column,p.row+row));
    }}
}

#[cfg(test)]
mod casino_interior_tests {
    use super::*;
    use std::sync::Arc;
    fn fixture(block: u16) -> (Vec<VisualTile>, GridGeometry) {
        let g = GridGeometry {
            width: 6,
            height: 6,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let mut tiles: Vec<VisualTile> = (0..36)
            .map(|i| VisualTile {
                column: (i % 6) as u32,
                row: (i / 6) as u32,
                texture: Handle::default(),
                priority: false,
                animation_frames: None,
                source: VisualTileSource {
                    tileset_id: Arc::from("game_corner"),
                    metatile_id: 1,
                    subtile_column: (i % 6 % 4) as u8,
                    subtile_row: (i / 6 % 4) as u8,
                    tile_index: 1,
                },
            })
            .collect();
        let drawing = casino_drawing(block).unwrap();
        for y in 0..4 {
            for x in 0..4 {
                let s = &mut tiles[y * 6 + x].source;
                s.metatile_id = block;
                s.subtile_column = x as u8;
                s.subtile_row = y as u8;
                s.tile_index = drawing[y * 4 + x];
            }
        }
        (tiles, g)
    }
    #[test]
    fn arcade_prize_rack_requires_whole_drawing_and_retains_dimensions_and_support() {
        for map in ["GoldenrodGameCorner", "CeladonGameCorner"] {
            let (tiles, g) = fixture(0x04);
            let refs = tiles.iter().collect::<Vec<_>>();
            let resolved = resolve(map, &refs, &g, None);
            let rack = resolved.iter().find(|p| p.kind == ModelKind::ArcadePrizeRack)
                .expect("native prize drawing must select the dedicated case");
            assert!(!resolved.iter().any(|p| p.kind == ModelKind::RetailShelf));
            assert_eq!((rack.width, rack.height), (4, 3));
            assert_eq!((rack.depth_pixels, rack.height_pixels), (10.0, 28.0));
            let mut mesh = TerrainMeshData::default();
            mesh.authored_cells = vec![None; tiles.len()];
            mesh.footing_heights = vec![72.0; tiles.len()];
            let mut claimed = vec![false; tiles.len()];
            assert!(append(&mut mesh, map, &refs, &g, rack, &mut claimed, [0, 0]));
            assert_eq!(claimed.iter().filter(|&&v| v).count(), 12);
            assert!(mesh.footing_heights.iter().all(|&v| v == 72.0));
            assert!(mesh.solid.positions.iter().all(|p|
                p[0] >= 0.0 && p[0] <= 32.0 && p[1] >= 0.0 && p[1] <= 28.001
                    && p[2] >= 14.0 && p[2] <= 24.0));
            assert!(mesh.authored_cells.iter().flatten()
                .all(|&label| label == "interior/arcade_prize_rack"));
            for y in 0..4 { for x in 0..4 { for field in 0..5 {
                let mut changed = tiles.clone();
                let s = &mut changed[y * g.width + x].source;
                match field {
                    0 => s.metatile_id ^= 0x80,
                    1 => s.tile_index ^= 0x80,
                    2 => s.subtile_column ^= 0x80,
                    3 => s.subtile_row ^= 0x80,
                    _ => s.tileset_id = Arc::from("unknown"),
                }
                let altered = resolve(map, &changed.iter().collect::<Vec<_>>(), &g, None);
                assert!(!altered.iter().any(|p| matches!(p.kind,
                    ModelKind::ArcadePrizeRack | ModelKind::RetailShelf)));
            } } }
            for i in (0..3).flat_map(|y| (0..4).map(move |x| y * g.width + x)) {
                let mut reserved = vec![false; tiles.len()]; reserved[i] = true;
                let before = reserved.clone(); let mut out = vec![];
                append_casino_candidates(map, &refs, &g, &mut reserved, &mut out);
                assert!(out.is_empty()); assert_eq!(reserved, before);
            }
        }
    }
    #[test]
    fn arcade_poster_has_closed_backing_and_keeps_source_ownership() {
        let (tiles, g) = fixture(0x28);
        let refs = tiles.iter().collect::<Vec<_>>();
        let pieces = resolve("CeladonGameCorner", &refs, &g, None);
        let mut mesh = TerrainMeshData::default();
        mesh.authored_cells = vec![None; tiles.len()];
        mesh.footing_heights = vec![72.0; tiles.len()];
        let mut claimed = vec![false; tiles.len()];
        for p in &pieces {
            assert!(append(&mut mesh, "CeladonGameCorner", &refs, &g, p, &mut claimed, [0, 0]));
        }
        let mut wall = SurfaceMeshData::default();
        model(ModelKind::ArcadeWall).append_fitted(
            &mut wall, [16.0, 32.0, 5.0, 8.0], 0.0, 32.0);
        assert!(mesh.solid.positions.windows(wall.positions.len())
            .any(|part| part == wall.positions.as_slice()));
        assert!(mesh.solid.positions.iter().any(|p| p[2] > 8.19));
        assert!(mesh.solid.positions.iter().all(|p| p[2] <= 8.201));
        assert_eq!(mesh.authored_cells.iter().filter(|&&v|
            v == Some("interior/arcade_wall")).count(), 4);
        assert_eq!(mesh.authored_cells.iter().filter(|&&v|
            v == Some("interior/picture_frame")).count(), 4);
        assert_eq!(claimed.iter().filter(|&&v| v).count(), 8);
        assert!(mesh.footing_heights.iter().all(|&v| v == 72.0));
    }
    #[test]
    fn arcade_wall_drawings_share_one_plane_without_duplicate_lower_courses() {
        for block in [2, 0x0d, 0x13, 0x1c, 0x28, 0x2b, 0x2f, 0x30] {
            let (tiles, g) = fixture(block);
            let refs = tiles.iter().collect::<Vec<_>>();
            let walls = resolve("GoldenrodGameCorner", &refs, &g, None)
                .into_iter().filter(|p| p.kind == ModelKind::ArcadeWall)
                .collect::<Vec<_>>();
            let mut mesh = TerrainMeshData::default();
            mesh.authored_cells = vec![None; tiles.len()];
            mesh.footing_heights = vec![72.0; tiles.len()];
            let mut claimed = vec![false; tiles.len()];
            for p in &walls {
                assert!(append(&mut mesh, "GoldenrodGameCorner", &refs, &g, p, &mut claimed, [0, 0]));
            }
            assert!(mesh.solid.positions.iter().all(|p|
                p[2] >= 5.0 - 0.001 && p[2] <= 8.0 + 0.001
                    && p[1] >= 0.0 && p[1] <= 32.0 + 0.001));
            assert_eq!(mesh.solid.positions.iter().map(|p|p[1]).fold(0.,f32::max),32.,
                "one ceiling datum must remain above29px rear appliances");
            let units = if block == 0x28 { 1 } else { 2 };
            let mut unit = SurfaceMeshData::default();
            model(ModelKind::ArcadeWall).append_fitted(
                &mut unit, [0.0, 16.0, 5.0, 8.0], 0.0, 32.0);
            assert_eq!(mesh.solid.positions.len(), unit.positions.len() * units);
            assert_eq!(claimed.iter().filter(|&&v| v).count(),
                walls.iter().map(|p| p.width * p.height).sum::<usize>());
            assert!(mesh.footing_heights.iter().all(|&v| v == 72.0));
            assert!(mesh.authored_cells.iter().flatten()
                .all(|&label| label == "interior/arcade_wall"));
        }
    }
    #[test]
    fn arcade_wall_uses_complete_native_course_without_acoustic_panel_fallback() {
        for map in ["GoldenrodGameCorner", "CeladonGameCorner"] {
            for block in [2, 0x0d] {
                let (tiles, g) = fixture(block);
                let refs = tiles.iter().collect::<Vec<_>>();
                let resolved = resolve(map, &refs, &g, None);
                assert_eq!(
                    resolved
                        .iter()
                        .filter(|p| p.kind == ModelKind::ArcadeWall)
                        .count(),
                    1
                );
                assert!(!resolved.iter().any(|p| p.kind == ModelKind::WallAcoustic));
                for y in 0..4 {
                    for x in 0..4 {
                        for mutation in 0..5 {
                            let mut changed = tiles.clone();
                            let s = &mut changed[y * g.width + x].source;
                            match mutation {
                                0 => s.metatile_id ^= 0x80,
                                1 => s.tile_index ^= 0x80,
                                2 => s.subtile_column ^= 0x80,
                                3 => s.subtile_row ^= 0x80,
                                _ => s.tileset_id = Arc::from("unknown"),
                            }
                            let resolved =
                                resolve(map, &changed.iter().collect::<Vec<_>>(), &g, None);
                            assert!(!resolved.iter().any(|p| matches!(
                                p.kind,
                                ModelKind::ArcadeWall | ModelKind::WallAcoustic
                            )));
                        }
                    }
                }
                for i in (0..2).flat_map(|y| (0..4).map(move |x| y * g.width + x)) {
                    let mut claimed = vec![false; 36];
                    claimed[i] = true;
                    let before = claimed.clone();
                    let mut out = vec![];
                    append_casino_candidates(map, &refs, &g, &mut claimed, &mut out);
                    assert!(out.is_empty());
                    assert_eq!(claimed, before);
                }
            }
        }
    }
    #[test]
    fn arcade_game_table_requires_both_halves_and_keeps_blue_board_and_four_legs() {
        let (seed, _) = fixture(0x2f);
        let mut g = GridGeometry {width: 10, height: 6, tile_width: 8., tile_height: 8.,
            origin_x: 0., origin_z: 0.};
        let mut tiles: Vec<_> = (0..60).map(|i| {
            let mut t=seed[35].clone();t.column=(i%10) as u32;t.row=(i/10) as u32;
            t.source.metatile_id=1;t.source.tile_index=1;
            t.source.subtile_column=(i%10%4) as u8;t.source.subtile_row=(i/10%4) as u8;t
        }).collect();
        for (offset,block) in [(0,0x2f),(4,0x30)] { for y in 0..4 { for x in 0..4 {
            let source=&mut tiles[y*10+offset+x].source;
            source.metatile_id=block;source.subtile_column=x as u8;source.subtile_row=y as u8;
            source.tile_index=casino_drawing(block).unwrap()[y*4+x];
        }}}
        let select=|tiles: &[VisualTile], g: &GridGeometry, reserved: Vec<bool>| {
            let mut claimed=reserved;let mut out=Vec::new();
            append_casino_candidates("GoldenrodGameCorner", &tiles.iter().collect::<Vec<_>>(),
                g,&mut claimed,&mut out);out
        };
        let pieces=select(&tiles,&g,vec![false;60]);
        let table=pieces.iter().find(|p|p.kind==ModelKind::DiningTable).unwrap();
        assert_eq!((table.column,table.row,table.width,table.height),(2,1,4,3));
        assert_eq!(table.indices(10).count(),12);
        assert_eq!(pieces.iter().filter(|p|p.kind==ModelKind::DiningTable).count(),1);
        assert!(!pieces.iter().any(|p|p.kind==ModelKind::ReceptionCounter));
        assert_eq!(pieces.iter().filter(|p|p.kind==ModelKind::ArcadeStool).count(),2);
        for scale in [0.5,1.,2.] {
            g.tile_width=8.*scale;g.tile_height=8.*scale;
            let mut mesh=TerrainMeshData {footing_heights:vec![7.;60],..Default::default()};
            assert!(append(&mut mesh,"GoldenrodGameCorner",&tiles.iter().collect::<Vec<_>>(),&g,table,
                &mut vec![false;60],[0,0]));
            assert!(mesh.solid.indices.is_empty());
            assert_eq!(mesh.footing_heights,vec![7.;60]);
            // Underlay: 12 quads; table: 10 closed boxes plus 8 board quads.
            assert_eq!(mesh.textured.indices.len(),12*6+10*36+8*6);
            assert!(mesh.textured.positions.iter().all(|v|v[0]>=16.*scale && v[0]<=48.*scale
                && v[2]>=8.*scale && v[2]<=32.*scale && v[1]>=0. && v[1]<=10.002*scale));
            let uv=g.uv(table.column+1,table.row);
            let blue=[uv.0+(uv.1-uv.0)*0.5/8.,uv.2+(uv.3-uv.2)*4.5/8.];
            assert!(mesh.textured.uvs.contains(&blue));
        }
        // Every native source cell, including both surrounding courses,
        // guards the compound drawing. Never reinterpret the surviving half.
        for y in 0..4 {for x in 0..8 {
            let mut changed=tiles.clone();changed[y*10+x].source.tile_index^=0x80;
            let after=select(&changed,&g,vec![false;60]);
            assert!(after.iter().all(|p|p.kind!=ModelKind::DiningTable
                && p.kind!=ModelKind::ReceptionCounter));
        }}
        for i in table.indices(10) {
            let mut reserved=vec![false;60];reserved[i]=true;
            assert!(select(&tiles,&g,reserved).iter().all(|p|p.kind!=ModelKind::DiningTable));
        }
        let cropped: Vec<_>=(0..6).flat_map(|y|tiles[y*10..y*10+7].iter().cloned()).collect();
        let cropped_g=GridGeometry {width:7,..g};
        assert!(select(&cropped,&cropped_g,vec![false;42]).iter().all(|p|
            p.kind!=ModelKind::DiningTable && p.kind!=ModelKind::ReceptionCounter));
    }

    #[test]
    fn casino_architecture_requires_complete_source_and_preserves_support() {
        for block in [
            2, 4, 9, 0x0d, 0x10, 0x11, 0x13, 0x14, 0x1b, 0x1c, 0x27, 0x28, 0x2b, 0x2f, 0x30,
        ] {
            let (mut t, g) = fixture(block);
            let mut claimed = vec![false; 36];
            let mut p = Vec::new();
            append_casino_candidates(
                "GoldenrodGameCorner",
                &t.iter().collect::<Vec<_>>(),
                &g,
                &mut claimed,
                &mut p,
            );
            assert!(!p.is_empty(), "block {block:x}");
            assert!(!claimed[4] && !claimed[24], "adjacent floor claimed");
            let mut mesh = TerrainMeshData {
                footing_heights: vec![0.0; 36],
                authored_cells: vec![None; 36],
                ..Default::default()
            };
            let mut drawn = vec![false; 36];
            for piece in &p {
                assert!(append(
                    &mut mesh,
                    "GoldenrodGameCorner",
                    &t.iter().collect::<Vec<_>>(),
                    &g,
                    piece,
                    &mut drawn,
                    [0, 0]
                ));
            }
            assert_eq!(drawn, claimed);
            assert_eq!(mesh.footing_heights, vec![0.0; 36]);
            assert!(!mesh.solid.positions.is_empty());
            t[0].source.tile_index ^= 0x80;
            let mut bad = Vec::new();
            append_casino_candidates(
                "GoldenrodGameCorner",
                &t.iter().collect::<Vec<_>>(),
                &g,
                &mut vec![false; 36],
                &mut bad,
            );
            assert!(bad.is_empty());
        }
    }
    #[test]
    fn casino_custom_claims_and_unrelated_maps_retain_ownership() {
        let (t, g) = fixture(9);
        let cells = t.iter().collect::<Vec<_>>();
        let mut claimed = vec![false; 36];
        claimed[12] = true;
        let mut p = Vec::new();
        append_casino_candidates("CeladonGameCorner", &cells, &g, &mut claimed, &mut p);
        assert!(p.is_empty());
        append_casino_candidates("VermilionGym", &cells, &g, &mut vec![false; 36], &mut p);
        assert!(p.is_empty());
    }
}
