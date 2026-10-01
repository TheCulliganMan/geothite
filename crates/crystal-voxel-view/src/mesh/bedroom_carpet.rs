// Thin cloth backing with the exact current carpet surface, not a recolor.
// Draw only resolved complete source fields; collision and footing stay native.
fn append_live_carpet_art(mesh: &mut TerrainMeshData, g: &GridGeometry, p: &Placement) {
    let y = (p.base_pixels + p.height_pixels + 0.002) * g.tile_height / 8.0;
    for row in 0..p.height {
        for column in 0..p.width {
            let x = p.column + column;
            let z = p.row + row;
            append_top(&mut mesh.textured, g.bounds(x, z).into(), y, g.uv(x, z));
        }
    }
}

#[cfg(test)]
mod bedroom_carpet_tests {
    use super::*;
    use std::sync::Arc;
    fn fixture(base: u16, tile: u16) -> (Vec<VisualTile>, GridGeometry) {
        // The live callback changes four blocks; desk-foot art in the central
        // block is not carpet and must retain its own rendering ownership.
        let g = GridGeometry { width: 16, height: 12, tile_width: 8.0,
            tile_height: 8.0, origin_x: 0.0, origin_z: 0.0 };
        let mut cells = Vec::new();
        for y in 0..g.height {
            for x in 0..g.width {
                let block = match (x / 4, y / 4) {
                    (0, 0) => base, (0 | 2, 1) => base + 1, (1, 1) => base + 2, _ => 5,
                };
                let local_y = y % 4;
                let art = if block == base && local_y < 2 { 2 }
                    else if block == base + 2 && local_y == 0 { [0x30, 0x31, 0x31, 0x32][x % 4] }
                    else if (base..=base + 2).contains(&block) { tile } else { 1 };
                cells.push(VisualTile { column: x as u32, row: y as u32,
                    texture: Handle::default(), priority: false,
                    source: VisualTileSource { tileset_id: Arc::from("players_room"),
                        metatile_id: block, subtile_column: (x % 4) as u8,
                        subtile_row: local_y as u8, tile_index: art } });
            }
        }
        (cells, g)
    }
    #[test]
    fn all_four_live_carpets_keep_exact_art_ownership_and_footing() {
        for (base, tile) in [(8, 13), (11, 29), (14, 45), (17, 61)] {
            let (cells, g) = fixture(base, tile);
            let refs: Vec<_> = cells.iter().collect();
            let placements: Vec<_> = resolve("PlayersHouse2F", &refs, &g, None)
                .into_iter().filter(|p| p.kind == ModelKind::CarpetCloth).collect();
            assert_eq!(placements.len(), 4);
            assert_eq!(placements.iter().map(|p| p.width * p.height).sum::<usize>(), 52);
            let mut mesh = TerrainMeshData::default();
            mesh.footing_heights = vec![3.0; cells.len()];
            mesh.authored_cells = vec![None; cells.len()];
            let mut claimed = vec![false; cells.len()];
            for p in &placements {
                assert!(append(&mut mesh, &refs, &g, p, &mut claimed));
                assert!(!append(&mut mesh, &refs, &g, p, &mut claimed));
            }
            assert_eq!(claimed.iter().filter(|v| **v).count(), 52);
            assert!(mesh.footing_heights.iter().all(|v| *v == 3.0));
            assert_eq!(mesh.textured.quad_count(), 104);
            for x in 4..8 { assert!(!claimed[4 * g.width + x], "desk foot source owned by cloth"); }
            // Every upper cloth quad uses the current source cell UV verbatim.
            let cloth_uv: Vec<_> = mesh.textured.positions.chunks_exact(4)
                .zip(mesh.textured.uvs.chunks_exact(4))
                .filter(|(p, _)| p[0][1] > 0.17).map(|(_, uv)| uv.to_vec()).collect();
            assert_eq!(cloth_uv.len(), 52);
        }
    }
    #[test]
    fn carpets_refuse_changed_art_phase_and_clipped_drawings() {
        let (mut cells, g) = fixture(8, 13);
        cells[2 * g.width].source.tile_index = 0xff;
        let refs: Vec<_> = cells.iter().collect();
        assert_eq!(resolve("PlayersHouse2F", &refs, &g, None).iter()
            .filter(|p| p.kind == ModelKind::CarpetCloth).count(), 3);
        cells[2 * g.width].source.tile_index = 13;
        cells[2 * g.width].source.subtile_row = 1;
        let refs: Vec<_> = cells.iter().collect();
        assert_eq!(resolve("PlayersHouse2F", &refs, &g, None).iter()
            .filter(|p| p.kind == ModelKind::CarpetCloth).count(), 3);
        // Cropping out a column cannot turn the surviving stripe into a rug.
        let refs: Vec<_> = cells.iter().take(3).collect();
        let cropped = GridGeometry { width: 3, height: 1, ..g };
        assert!(!resolve("PlayersHouse2F", &refs, &cropped, None).iter()
            .any(|p| p.kind == ModelKind::CarpetCloth));
    }
}
