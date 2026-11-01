//! Authentic snow backing for the imported Falls forest drawing.
use super::{CellShape, GridGeometry, TreePlacement, VisualTile};

pub(super) fn sample(
    map: &str,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    geometry: &GridGeometry,
    tree: TreePlacement,
) -> Option<usize> {
    let source = &cells[tree.row * geometry.width + tree.column].source;
    if map != "MnInternationalFalls"
        || source.tileset_id.as_ref() != "forest"
        || source.metatile_id != 0x05
    {
        return None;
    }
    // This private block is the imported winter snow floor. Borrow its real
    // texture from this frame; never relabel it as native forest lawn or use
    // a different atlas's numerically coincident glyph.
    cells.iter().zip(shapes).position(|(tile, shape)| {
        tile.source.tileset_id.as_ref() == "mn_international_falls"
            && tile.source.metatile_id == 0x80
            && matches!(shape, CellShape::Flat | CellShape::PlaneAt { height: 0.0 })
    })
}

#[cfg(test)]
mod tests {
    use super::super::{TerrainMeshData, append_tree, complete_tree_placements, shape_for_source};
    use super::{GridGeometry, sample};

    #[test]
    fn falls_snow_builds_a_solid_canopy_without_borrowing_unrelated_ground() {
        let sources = (0..4)
            .flat_map(|row| {
                (0..5).map(move |column| {
                    let canopy = column < 4;
                    let mut source = super::super::super::tests::source_with_tile(
                        if canopy { 0x05 } else { 0x80 },
                        column,
                        row,
                        if canopy {
                            0x0c + u16::from(row) * 0x10 + u16::from(column)
                        } else {
                            0x05
                        },
                    );
                    source.tileset_id = if canopy {
                        "forest"
                    } else {
                        "mn_international_falls"
                    }
                    .into();
                    source
                })
            })
            .collect();
        let mut frame = super::super::super::tests::frame(5, 4, sources);
        let geometry = GridGeometry {
            width: 5,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells
            .iter()
            .map(|tile| shape_for_source(&tile.source))
            .collect();
        let tree = complete_tree_placements(&cells, &geometry)[0];
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.0; 20],
            authored_cells: vec![None; 20],
            ..Default::default()
        };
        let mut claimed = vec![false; 20];
        assert!(append_tree(
            &mut mesh,
            &cells,
            &shapes,
            &geometry,
            "MnInternationalFalls",
            tree,
            &mut claimed,
            [0, 0]
        ));
        assert!(
            mesh.solid
                .positions
                .iter()
                .any(|position| position[1] > 8.0)
        );
        assert_eq!(mesh.textured.quad_count(), 16);
        assert_eq!(
            claimed,
            (0..20).map(|index| index % 5 < 4).collect::<Vec<_>>()
        );
        assert!(sample("IlexForest", &cells, &shapes, &geometry, tree).is_none());
        for tile in &mut frame.tiles {
            if tile.source.metatile_id == 0x80 {
                tile.source.metatile_id = 0x7f;
            }
        }
        let cells: Vec<_> = frame.tiles.iter().collect();
        assert!(sample("MnInternationalFalls", &cells, &shapes, &geometry, tree).is_none());
        let mut refused = TerrainMeshData::default();
        assert!(!append_tree(
            &mut refused,
            &cells,
            &shapes,
            &geometry,
            "MnInternationalFalls",
            tree,
            &mut [false; 20],
            [0, 0]
        ));
        assert!(refused.solid.positions.is_empty());
    }
}
