//! Complete native mountain entrance drawings, with an open recessed mouth.
//! The source still owns footing and warps; this only replaces their artwork.
use super::*;

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    claimed: &mut [bool],
    origin: [i32; 2],
) {
    for (i, cell) in cells.iter().enumerate() {
        let s = &cell.source;
        let (column, art) = match s.tileset_id.as_ref() {
            "johto" => (0, [[0x46, 0x47], [0x56, 0x57]]),
            "johto_modern" => (2, [[0x45, 0x22], [0x27, 0x28]]),
            _ => continue,
        };
        if s.metatile_id != 0x73 || s.subtile_column != column || s.subtile_row != 2 {
            continue;
        }
        let (x, y) = (i % g.width, i / g.width);
        if x + 2 > g.width || y + 2 > g.height {
            continue;
        }
        let complete = (0..2).all(|dy| {
            (0..2).all(|dx| {
                let at = (y + dy) * g.width + x + dx;
                let other = &cells[at].source;
                !claimed[at]
                    && other.tileset_id == s.tileset_id
                    && other.metatile_id == s.metatile_id
                    && usize::from(other.subtile_column) == usize::from(column) + dx
                    && usize::from(other.subtile_row) == 2 + dy
                    && other.tile_index == art[dy][dx]
            })
        });
        if !complete {
            continue;
        }
        let (w, _, n, _) = g.bounds(x, y);
        let (_, e, _, south) = g.bounds(x + 1, y + 1);
        // Use the same resolved local mountain datum, including stacked tiers.
        let mut h = g.tile_height * crate::profile::MOUNTAIN_CLIFF_HEIGHT / SOURCE_TILE_HEIGHT;
        for row in y.saturating_sub(1)..=(y + 1).min(g.height - 1) {
            for col in x.saturating_sub(1)..=(x + 2).min(g.width - 1) {
                if matches!(
                    shapes[row * g.width + col],
                    CellShape::RaisedTop {
                        solid: SolidKind::Bank,
                        ..
                    } | CellShape::LedgeBand { .. }
                ) {
                    h = h.max(shapes[row * g.width + col].surface_height(g.tile_height));
                }
            }
        }
        let t = g.tile_height;
        let side = t * 0.20;
        rock_block(&mut mesh.solid, [w, w + side, 0., h, n, south], g, origin);
        rock_block(&mut mesh.solid, [e - side, e, 0., h, n, south], g, origin);
        let span = e - w - side * 2.;
        let arch = [
            (0., 0.62),
            (0.22, 0.80),
            (0.5, 0.88),
            (0.78, 0.80),
            (1., 0.62),
        ];
        for segment in arch.windows(2) {
            let (x0, y0) = (w + side + span * segment[0].0, h * segment[0].1);
            let (x1, y1) = (w + side + span * segment[1].0, h * segment[1].1);
            for points in [
                [
                    [x0, y0, south],
                    [x1, y1, south],
                    [x1, h, south],
                    [x0, h, south],
                ],
                [[x1, y1, n], [x0, y0, n], [x0, h, n], [x1, h, n]],
                [[x0, h, n], [x0, h, south], [x1, h, south], [x1, h, n]],
                [[x0, y0, south], [x0, y0, n], [x1, y1, n], [x1, y1, south]],
            ] {
                let p = points.map(Vec3::from_array);
                let normal = (p[1] - p[0]).cross(p[3] - p[0]).normalize().to_array();
                outdoor_stone::append_canyon(&mut mesh.solid, points, normal, g, origin);
            }
        }
        // Only the recessed vertical wall is dark. A closed dark cuboid
        // exposes its top above the mineral-deformed arch as a black cap seam.
        // Keep its upper edge inside the recess too: the mineral finish moves
        // the arch's intrados, while this vertical dark wall stays planar.
        for segment in arch.windows(2) {
            let x0 = w + side + span * segment[0].0;
            let x1 = w + side + span * segment[1].0;
            append_quad(
                &mut mesh.solid,
                [
                    [x0, 0., n + t * 0.10],
                    [x1, 0., n + t * 0.10],
                    [x1, (h * segment[1].1 - t * 0.30).max(0.), n + t * 0.10],
                    [x0, (h * segment[0].1 - t * 0.30).max(0.), n + t * 0.10],
                ],
                [0., 0., 1.],
                [[0., 0.]; 4],
                [0.025, 0.026, 0.030, 1.],
            );
        }
        super::modeled_exteriors::portal_block(
            &mut mesh.solid,
            [w + side, e - side, 0., t * 0.015, n, south],
            [0.19, 0.17, 0.12],
        );
        for dy in 0..2 {
            for dx in 0..2 {
                let at = (y + dy) * g.width + x + dx;
                claimed[at] = true;
                mesh.authored_cells[at] = Some("johto/mountain_cave");
            }
        }
    }
}

fn rock_block(out: &mut SurfaceMeshData, bounds: [f32; 6], g: &GridGeometry, origin: [i32; 2]) {
    let mut block = SurfaceMeshData::default();
    super::modeled_exteriors::portal_block(&mut block, bounds, [0.49, 0.36, 0.20]);
    for (p, n) in block
        .positions
        .chunks_exact(4)
        .zip(block.normals.chunks_exact(4))
    {
        outdoor_stone::append_canyon(out, p.try_into().unwrap(), n[0], g, origin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_portals_replace_four_source_cells_without_changing_footing() {
        for (tileset, first_column, art) in [
            ("johto", 0, [[0x46, 0x47], [0x56, 0x57]]),
            ("johto_modern", 2, [[0x45, 0x22], [0x27, 0x28]]),
        ] {
            let mut sources = vec![super::super::tests::source_with_tile(1, 0, 0, 6); 16];
            for y in 0..2 {
                for x in 0..2 {
                    let mut source = super::super::tests::source_with_tile(
                        0x73,
                        first_column + x as u8,
                        2 + y as u8,
                        art[y][x],
                    );
                    source.tileset_id = tileset.into();
                    sources[(y + 1) * 4 + x + 1] = source;
                }
            }
            let mut frame = super::super::tests::frame(4, 4, sources);
            let g = GridGeometry {
                width: 4,
                height: 4,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: -16.,
                origin_z: -16.,
            };
            let mut mesh = TerrainMeshData::default();
            mesh.footing_heights = vec![7.; 16];
            mesh.authored_cells = vec![None; 16];
            let mut claimed = vec![false; 16];
            let cells = frame.tiles.iter().collect::<Vec<_>>();
            let shapes = cells
                .iter()
                .map(|cell| shape_for_source(&cell.source))
                .collect::<Vec<_>>();
            append(&mut mesh, &cells, &shapes, &g, &mut claimed, [0, 0]);
            assert_eq!(mesh.footing_heights, vec![7.; 16]);
            assert_eq!(claimed.iter().filter(|v| **v).count(), 4);
            assert!(mesh.textured.positions.is_empty());
            assert!(!mesh.solid.indices.is_empty());
            for (color, normal) in mesh.solid.colors.iter().zip(&mesh.solid.normals) {
                if color[0] < 0.03 && color[1] < 0.03 {
                    assert_eq!(
                        normal[1], 0.,
                        "recess darkness must not expose a horizontal cap"
                    );
                }
            }
            frame.tiles[10].source.tile_index ^= 1;
            let mut rejected = TerrainMeshData::default();
            append(
                &mut rejected,
                &frame.tiles.iter().collect::<Vec<_>>(),
                &shapes,
                &g,
                &mut vec![false; 16],
                [0, 0],
            );
            assert!(
                rejected.solid.positions.is_empty(),
                "a changed or partial source drawing stays untouched"
            );
        }
    }
}
