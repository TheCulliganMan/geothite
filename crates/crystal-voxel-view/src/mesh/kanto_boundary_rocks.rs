//! Complete Kanto boundary drawings replaced by original closed stone meshes.
//!
//! The existing classifiers own source identity and drawing size. This adapter
//! additionally requires a coherent native block and the correct live underlay.
//! It never reads collision and never changes the original footing heights.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct Placement {
    pub(super) rect: [usize; 4],
    sample: usize,
    kind: Kind,
    base: f32,
}

fn coherent(cells: &[&VisualTile], g: &GridGeometry, p: TreePlacement) -> bool {
    let first = &cells[p.row * g.width + p.column].source;
    // Every family occupies one quadrant of one native 4x4 block. Modulo-2
    // classification alone would also admit an unrelated allowed block or a
    // source quadrant moved by two cells into this drawing.
    (0..p.height).all(|y| {
        (0..p.width).all(|x| {
            let source = &cells[(p.row + y) * g.width + p.column + x].source;
            source.tileset_id == first.tileset_id
                && source.metatile_id == first.metatile_id
                && usize::from(source.subtile_column) == usize::from(first.subtile_column) + x
                && usize::from(source.subtile_row) == usize::from(first.subtile_row) + y
        })
    })
}

pub(super) fn resolve(
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
) -> Vec<Placement> {
    let mut placements = Vec::new();
    for (kind, groups) in [
        (
            Kind::KantoBoundaryLand,
            kanto_round_barrier_placements(cells, g),
        ),
        (
            Kind::KantoBoundaryShore,
            kanto_shoreline_round_barrier_placements(cells, g),
        ),
        (
            Kind::KantoBoundaryPath,
            kanto_round_path_barrier_placements(cells, g),
        ),
    ] {
        for p in groups {
            if !coherent(cells, g, p) {
                continue;
            }
            let base = p.base_height * g.tile_height / SOURCE_TILE_HEIGHT;
            let Some(sample) = cells.iter().zip(shapes).position(|(cell, shape)| {
                cell.source.tileset_id.as_ref() == "kanto"
                    && cell.source.tile_index == p.ground_tile_index
                    && matches!(
                        shape,
                        CellShape::Flat
                            | CellShape::Water
                            | CellShape::PlaneAt { .. }
                            | CellShape::RaisedTop {
                                solid: SolidKind::Bank,
                                ..
                            }
                    )
                    && (shape.surface_height(g.tile_height) - base).abs() < 0.001
            }) else {
                continue;
            };
            placements.push(Placement {
                rect: [p.column, p.row, p.width, p.height],
                sample,
                kind,
                base,
            });
        }
    }
    placements
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    claimed: &mut [bool],
) {
    for p in resolve(cells, shapes, g) {
        if !clear(claimed, g, p.rect) {
            continue;
        }
        // floor uses this exact sample's datum, including recessed animated
        // water. All four consumed source cells retain an actual native top.
        floor(mesh, cells, shapes, g, p.rect, p.sample, claimed);
        let [west, east, north, south] = bounds(g, p.rect);
        let fitted = [
            west + g.tile_width * 0.06,
            east - g.tile_width * 0.06,
            north + g.tile_height * 0.09,
            south - g.tile_height * 0.09,
        ];
        let m = model(p.kind);
        // Shared, cached material meshes append to the existing single solid
        // terrain batch. No per-rock render entities, image copies or textures.
        let rise = g.tile_height
            * if p.kind == Kind::KantoBoundaryPath {
                1.45
            } else {
                1.55
            };
        m.append_fitted(
            &mut mesh.solid,
            fitted,
            p.base,
            rise / (m.max[1] - m.min[1]),
            None,
        );
        mark_authored_rect(mesh, g, p.rect, p.kind.label());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn fixture(kind: Kind, tile_height: f32) -> (Vec<VisualTile>, Vec<CellShape>, GridGeometry) {
        let (block, column, ground_tile, ground_shape) = match kind {
            Kind::KantoBoundaryLand => (0x4d, 2, 0x2c, CellShape::Flat),
            Kind::KantoBoundaryShore => (0x19, 2, 0x14, CellShape::Water),
            Kind::KantoBoundaryPath => (0x29, 2, 0x11, CellShape::Flat),
            _ => unreachable!(),
        };
        let mut cells = Vec::new();
        let mut shapes = Vec::new();
        for row in 0..2 {
            for x in 0..3 {
                let ground = x == 2;
                cells.push(VisualTile {
                    column: x as u32,
                    row: row as u32,
                    source: VisualTileSource {
                        tileset_id: Arc::from("kanto"),
                        metatile_id: if ground { 0x01 } else { block },
                        subtile_column: if ground { 0 } else { column + x as u8 },
                        subtile_row: row as u8,
                        tile_index: if ground {
                            ground_tile
                        } else if kind == Kind::KantoBoundaryPath {
                            0x24
                        } else {
                            [[0x2a, 0x2b], [0x3a, 0x3b]][row][x]
                        },
                    },
                    texture: Handle::default(),
                    priority: false,
                });
                shapes.push(if ground {
                    ground_shape
                } else {
                    CellShape::Flat
                });
            }
        }
        (
            cells,
            shapes,
            GridGeometry {
                width: 3,
                height: 2,
                tile_width: 8.0,
                tile_height,
                origin_x: -24.0,
                origin_z: 40.0,
            },
        )
    }

    fn kinds() -> [Kind; 3] {
        [
            Kind::KantoBoundaryLand,
            Kind::KantoBoundaryShore,
            Kind::KantoBoundaryPath,
        ]
    }

    #[test]
    fn complete_boundary_drawings_append_closed_geometry_and_preserve_footing() {
        for kind in kinds() {
            let (tiles, shapes, g) = fixture(kind, 8.0);
            let cells: Vec<_> = tiles.iter().collect();
            let before = tiles.iter().map(|t| t.source.clone()).collect::<Vec<_>>();
            let mut mesh = TerrainMeshData {
                footing_heights: vec![3.25; 6],
                authored_cells: vec![None; 6],
                ..Default::default()
            };
            let mut claimed = vec![false; 6];
            // Exercise the integrated exterior entry point, not only the resolver.
            append_props(&mut mesh, &cells, &shapes, &g, &mut claimed);
            assert_eq!(claimed, [true, true, false, true, true, false]);
            assert_eq!(mesh.footing_heights, [3.25; 6]);
            assert_eq!(mesh.textured.indices.len(), 4 * 6);
            assert_eq!(mesh.solid.indices.len(), model(kind).surface.indices.len());
            assert_eq!(
                mesh.authored_cells
                    .iter()
                    .filter(|v| **v == Some(kind.label()))
                    .count(),
                4
            );
            assert_eq!(
                before,
                tiles.iter().map(|t| t.source.clone()).collect::<Vec<_>>()
            );
            assert!(
                mesh.solid
                    .positions
                    .iter()
                    .all(|v| v[0] >= -24.0 && v[0] <= -8.0 && v[2] >= 40.0 && v[2] <= 56.0)
            );
            let y = if kind == Kind::KantoBoundaryShore {
                crate::profile::WATER_HEIGHT
            } else {
                0.0
            };
            assert!(
                mesh.textured
                    .positions
                    .iter()
                    .all(|v| (v[1] - y).abs() < 0.001)
            );
            assert!(mesh.solid.positions.iter().all(|v| v[1] >= y - 0.001));
            assert!(
                mesh.solid
                    .positions
                    .iter()
                    .any(|v| (v[1] - y).abs() < 0.001)
            );
            assert!(mesh.solid.positions.iter().any(|v| v[1] > y + 10.0));
            // A second append cannot double draw or double claim this rock.
            let count = mesh.solid.indices.len();
            append_props(&mut mesh, &cells, &shapes, &g, &mut claimed);
            assert_eq!(mesh.solid.indices.len(), count);
        }
    }

    #[test]
    fn cropped_changed_phase_and_mixed_drawings_keep_fallback() {
        for kind in kinds() {
            for change in 0..5 {
                let (mut tiles, shapes, mut g) = fixture(kind, 8.0);
                match change {
                    0 => tiles[4].source.tile_index ^= 1,
                    1 => tiles[4].source.subtile_column -= 2,
                    2 => tiles[4].source.subtile_row += 2,
                    3 => {
                        tiles[4].source.metatile_id = if kind == Kind::KantoBoundaryShore {
                            0x14
                        } else {
                            0x13
                        }
                    }
                    4 => {
                        g.height = 1;
                        tiles.truncate(3);
                    }
                    _ => unreachable!(),
                }
                let cells = tiles.iter().collect::<Vec<_>>();
                assert!(
                    resolve(&cells, &shapes[..tiles.len()], &g).is_empty(),
                    "{kind:?} change {change}"
                );
            }
        }
    }

    #[test]
    fn only_native_matching_datum_underlays_reserve_source_cells() {
        for kind in kinds() {
            let (mut tiles, mut shapes, g) = fixture(kind, 8.0);
            for i in [2, 5] {
                tiles[i].source.tileset_id = Arc::from("johto");
            }
            assert!(resolve(&tiles.iter().collect::<Vec<_>>(), &shapes, &g).is_empty());
            for i in [2, 5] {
                tiles[i].source.tileset_id = Arc::from("kanto");
                shapes[i] = CellShape::RaisedTop {
                    height: 16.0,
                    solid: SolidKind::Bank,
                };
            }
            assert!(resolve(&tiles.iter().collect::<Vec<_>>(), &shapes, &g).is_empty());
            for i in [2, 5] {
                tiles[i].source.tile_index = 0xffff;
                shapes[i] = CellShape::Flat;
            }
            let cells = tiles.iter().collect::<Vec<_>>();
            assert!(resolve(&cells, &shapes, &g).is_empty());
            assert!(!preferred_cells("Route17", &cells, &g).iter().any(|v| *v));
        }
    }

    #[test]
    fn shoreline_water_and_solid_use_the_same_scaled_datum() {
        let (tiles, shapes, g) = fixture(Kind::KantoBoundaryShore, 16.0);
        let cells = tiles.iter().collect::<Vec<_>>();
        let p = resolve(&cells, &shapes, &g)[0];
        assert_eq!(p.base, -4.0);
        let mut mesh = TerrainMeshData {
            authored_cells: vec![None; 6],
            ..Default::default()
        };
        append(&mut mesh, &cells, &shapes, &g, &mut [false; 6]);
        assert!(mesh.textured.positions.iter().all(|v| v[1] == -4.0));
        assert!(mesh.solid.positions.iter().all(|v| v[1] >= -4.0));
        assert!(mesh.solid.positions.iter().any(|v| v[1] == -4.0));
    }

    #[test]
    fn preferred_cells_and_append_agree_and_existing_claims_win() {
        for kind in kinds() {
            let (tiles, shapes, g) = fixture(kind, 8.0);
            let cells = tiles.iter().collect::<Vec<_>>();
            assert_eq!(
                preferred_cells("Route17", &cells, &g),
                [true, true, false, true, true, false]
            );
            let mut mesh = TerrainMeshData {
                authored_cells: vec![None; 6],
                ..Default::default()
            };
            let mut claimed = [false; 6];
            claimed[4] = true;
            append(&mut mesh, &cells, &shapes, &g, &mut claimed);
            assert!(mesh.solid.indices.is_empty());
            assert!(mesh.textured.indices.is_empty());
            assert!(!mesh.authored_cells.iter().any(Option::is_some));
            assert_eq!(claimed, [false, false, false, false, true, false]);
        }
    }
}
