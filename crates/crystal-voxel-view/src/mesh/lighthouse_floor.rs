// Muted flagstones for exact native lighthouse walkway cells. This is a
// surface finish, never a new object, collision shape, or footing height.
fn lighthouse_floor_source(map: &str, source: &VisualTileSource) -> bool {
    if !matches!(
        map,
        "OlivineLighthouse1F"
            | "OlivineLighthouse2F"
            | "OlivineLighthouse3F"
            | "OlivineLighthouse4F"
            | "OlivineLighthouse5F"
            | "OlivineLighthouse6F"
    ) || source.tileset_id.as_ref() != "lighthouse"
        || source.subtile_column >= 4
        || source.subtile_row >= 4
    {
        return false;
    }
    // A source-cell mask, not a collision/map-name recolor. $28's pit, $2e's
    // warp carpet and the southeast ladder drawings in $31/$3a are excluded.
    // The distinct $0d/$1d checker chamber on 6F is a separate material scope.
    let mask: u16 = match source.metatile_id {
        0x27 => 0xffff,
        0x28 | 0x2e => 0x00ff,
        0x31 | 0x3a => 0x33ff,
        _ => return false,
    };
    let x = source.subtile_column;
    let y = source.subtile_row;
    mask & (1 << (y * 4 + x)) != 0
        && source.tile_index == if (x + y) % 2 == 0 { 0x2e } else { 0x2f }
}

fn lighthouse_floor_destination(
    mesh: &TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    destination: usize,
) -> bool {
    let source = &cells[destination].source;
    if lighthouse_floor_source(map, source) {
        return true;
    }
    // The masonry resolver restores $27 beneath successfully appended walls.
    // A floor UV alone cannot authorize repainting a pit, prop, unknown source
    // or legacy relief. Only the exact successful masonry label is evidence.
    source.tileset_id.as_ref() == "lighthouse"
        && matches!(source.metatile_id, 0x29 | 0x3c | 0x3d | 0x3e)
        && matches!(
            mesh.authored_cells.get(destination),
            Some(Some(
                "dungeon:lighthouse-masonry" | "dungeon:lighthouse-window-masonry"
            ))
        )
}

fn lighthouse_floor_cell(
    mesh: &mut SurfaceMeshData,
    bounds: [f32; 4],
    height: f32,
    world: [i32; 2],
) {
    let [x0, x1, z0, z1] = bounds;
    let [column, row] = world;
    let course = row.div_euclid(2);
    let phase = course.rem_euclid(2) * 2;
    let slab = (column - phase).div_euclid(4);
    let base = InteriorFloor::LighthouseSlate.palette();
    let variation = ((slab * 3 + course * 5).rem_euclid(5) as f32 - 2.0) * 0.006;
    let face = tint(base, variation);
    let joint = tint(base, -0.055);
    // Broad 4x2-cell slabs with half-width staggered joints. Each quad exactly
    // partitions the original surface at its existing height: no raised grout,
    // overlaps, displacement, noise texture, extra entity or per-frame work.
    let west = if (column - phase).rem_euclid(4) == 0 {
        (x1 - x0) * 0.025
    } else {
        0.0
    };
    let north = if row.rem_euclid(2) == 0 {
        (z1 - z0) * 0.025
    } else {
        0.0
    };
    material_quad(mesh, [x0, x1, z0, z0 + north], height, joint);
    material_quad(mesh, [x0, x0 + west, z0 + north, z1], height, joint);
    material_quad(mesh, [x0 + west, x1, z0 + north, z1], height, face);
}

#[cfg(test)]
mod lighthouse_floor_tests {
    use super::*;
    use std::sync::Arc;

    const MAP: &str = "OlivineLighthouse4F";
    fn source(block: u16, x: u8, y: u8, tile: u16) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from("lighthouse"),
            metatile_id: block,
            subtile_column: x,
            subtile_row: y,
            tile_index: tile,
        }
    }
    fn grid() -> GridGeometry {
        GridGeometry {
            width: 2,
            height: 1,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -8.0,
            origin_z: -4.0,
        }
    }
    fn tiles(destination: VisualTileSource) -> Vec<VisualTile> {
        [source(0x27, 0, 0, 0x2e), destination]
            .into_iter()
            .enumerate()
            .map(|(i, source)| VisualTile {
                column: i as _,
                row: 0,
                texture: Handle::default(),
                priority: false,
                source,
            })
            .collect()
    }
    fn mesh(g: &GridGeometry, sample: usize) -> TerrainMeshData {
        let mut mesh = TerrainMeshData::default();
        append_top(
            &mut mesh.textured,
            [0.0, 8.0, -4.0, 4.0],
            3.5,
            g.uv(sample, 0),
        );
        mesh.footing_heights = vec![0.0, 3.5];
        mesh.authored_cells = vec![None, None];
        mesh
    }

    #[test]
    fn lighthouse_floor_exact_source_masks_preserve_navigation_and_custom_art() {
        // Independent accepted counts for each native mixed drawing.
        for (block, count) in [(0x27, 16), (0x28, 8), (0x2e, 8), (0x31, 12), (0x3a, 12)] {
            let mut accepted = 0;
            for y in 0..4 {
                for x in 0..4 {
                    let tile = if (x + y) % 2 == 0 { 0x2e } else { 0x2f };
                    let s = source(block, x, y, tile);
                    let selected = lighthouse_floor_source(MAP, &s);
                    accepted += usize::from(selected);
                    if (matches!(block, 0x28 | 0x2e) && y >= 2)
                        || (matches!(block, 0x31 | 0x3a) && x >= 2 && y >= 2)
                    {
                        assert!(!selected, "navigation drawings cannot become stone");
                    }
                    let mut changed = s.clone();
                    changed.tile_index = if tile == 0x2e { 0x2f } else { 0x2e };
                    assert!(!lighthouse_floor_source(MAP, &changed));
                    changed = s.clone();
                    changed.tileset_id = Arc::from("custom_lighthouse");
                    assert!(!lighthouse_floor_source(MAP, &changed));
                    assert!(!lighthouse_floor_source("FastShip1F", &s));
                    assert!(!lighthouse_floor_source("OlivineLighthouse7F", &s));
                }
            }
            assert_eq!(accepted, count);
        }
        for block in [
            0x06, 0x08, 0x0b, 0x0c, 0x29, 0x2d, 0x2f, 0x34, 0x36, 0x3c, 0x3d, 0x3e, 0xff,
        ] {
            for y in 0..4 {
                for x in 0..4 {
                    for tile in [0x01, 0x0d, 0x1d, 0x2e, 0x2f, 0x24, 0x34] {
                        assert!(!lighthouse_floor_source(MAP, &source(block, x, y, tile)));
                    }
                }
            }
        }
        assert!(!lighthouse_floor_source(MAP, &source(0x27, 4, 0, 0x2e)));
        assert!(!lighthouse_floor_source(MAP, &source(0x27, 0, 4, 0x2e)));
        for floor in 1..=6 {
            assert!(lighthouse_floor_source(
                &format!("OlivineLighthouse{floor}F"),
                &source(0x27, 0, 0, 0x2e)
            ));
        }
    }

    #[test]
    fn lighthouse_floor_uses_sampled_source_and_requires_masonry_for_underlays() {
        let g = grid();
        let wall = source(0x3c, 0, 0, 0x5e);
        let cells = tiles(wall);
        let refs: Vec<_> = cells.iter().collect();
        let mut missing_model = mesh(&g, 0);
        let before = missing_model.clone();
        assert_eq!(
            finish_surfaces(&mut missing_model, MAP, &refs, &g, [0, 0]),
            0
        );
        assert_eq!(missing_model, before);
        for label in [
            "dungeon:lighthouse-masonry",
            "dungeon:lighthouse-window-masonry",
        ] {
            let mut owned = mesh(&g, 0);
            owned.authored_cells[1] = Some(label);
            let footing = owned.footing_heights.clone();
            let authorship = owned.authored_cells.clone();
            assert_eq!(finish_surfaces(&mut owned, MAP, &refs, &g, [0, 0]), 1);
            assert!(owned.textured.indices.is_empty());
            assert_eq!(owned.footing_heights, footing);
            assert_eq!(owned.authored_cells, authorship);
            assert!(owned.solid.positions.iter().all(|p| p[1] == 3.5));
            // Sampling the wall itself, or an incomplete atlas rectangle, does
            // not gain floor permission from its successful-model destination.
            let mut wrong_sample = mesh(&g, 1);
            wrong_sample.authored_cells[1] = Some(label);
            let before = wrong_sample.clone();
            assert_eq!(
                finish_surfaces(&mut wrong_sample, MAP, &refs, &g, [0, 0]),
                0
            );
            assert_eq!(wrong_sample, before);
            let mut partial_uv = mesh(&g, 0);
            partial_uv.authored_cells[1] = Some(label);
            partial_uv.textured.uvs[0][0] += 0.1;
            partial_uv.textured.uvs[1][0] += 0.1;
            let before = partial_uv.clone();
            assert_eq!(finish_surfaces(&mut partial_uv, MAP, &refs, &g, [0, 0]), 0);
            assert_eq!(partial_uv, before);
        }
        // Even floor-sampled geometry and an unrelated successful prop label
        // cannot authorize repainting protected/unknown source destinations.
        for dest in [
            source(0x28, 0, 2, 0x01),
            source(0x2e, 0, 2, 0x24),
            source(0x31, 2, 2, 0x27),
            source(0x36, 2, 0, 0x46),
            source(0xff, 0, 0, 0x2e),
        ] {
            let cells = tiles(dest);
            let refs: Vec<_> = cells.iter().collect();
            let mut rejected = mesh(&g, 0);
            rejected.authored_cells[1] = Some("interior/unknown");
            let before = rejected.clone();
            assert_eq!(finish_surfaces(&mut rejected, MAP, &refs, &g, [0, 0]), 0);
            assert_eq!(rejected, before);
        }
    }

    #[test]
    fn lighthouse_floor_finish_preserves_geometry_footing_and_wall_cutaway_ranges() {
        let g = grid();
        let cells = tiles(source(0x27, 1, 0, 0x2f));
        let refs: Vec<_> = cells.iter().collect();
        let mut finished = mesh(&g, 1);
        // A preexisting solid wall must keep its exact buffers and cutaway mask.
        append_quad(
            &mut finished.solid,
            [
                [-8.0, 0.0, -4.0],
                [-8.0, 16.0, -4.0],
                [0.0, 16.0, -4.0],
                [0.0, 0.0, -4.0],
            ],
            [0.0, 0.0, -1.0],
            [[0.0; 2]; 4],
            [0.5; 4],
        );
        finished.solid.cutaway_ranges.push(0..4);
        let before = finished.clone();
        assert_eq!(finish_surfaces(&mut finished, MAP, &refs, &g, [-5, 8]), 1);
        assert_eq!(&finished.solid.positions[..4], &before.solid.positions);
        assert_eq!(&finished.solid.normals[..4], &before.solid.normals);
        assert_eq!(&finished.solid.uvs[..4], &before.solid.uvs);
        assert_eq!(&finished.solid.colors[..4], &before.solid.colors);
        assert_eq!(&finished.solid.indices[..6], &before.solid.indices);
        assert_eq!(finished.solid.cutaway_ranges, before.solid.cutaway_ranges);
        assert_eq!(finished.footing_heights, before.footing_heights);
        assert_eq!(finished.authored_cells, before.authored_cells);
        assert_eq!(finished.animated_solid, before.animated_solid);
        assert_eq!(finished.animated_textured, before.animated_textured);
        let mut area = 0.0;
        for quad in finished.solid.positions[4..].chunks_exact(4) {
            assert!(quad.iter().all(|p| p[1] == 3.5
                && (0.0..=8.0).contains(&p[0])
                && (-4.0..=4.0).contains(&p[2])));
            area += (quad[3][0] - quad[0][0]) * (quad[1][2] - quad[0][2]);
        }
        assert!((area - 64.0).abs() < 0.0001);
        assert!(
            finished.solid.normals[4..]
                .iter()
                .all(|n| *n == [0.0, 1.0, 0.0])
        );
        assert!(finished.solid.uvs[4..].iter().all(|uv| *uv == [0.0; 2]));
    }

    #[test]
    fn lighthouse_floor_slab_phase_is_stable_across_crops_and_negative_origins() {
        for row in -5..6 {
            for column in -7..8 {
                let mut a = SurfaceMeshData::default();
                let mut b = SurfaceMeshData::default();
                lighthouse_floor_cell(&mut a, [0.0, 8.0, 0.0, 8.0], 1.25, [column, row]);
                lighthouse_floor_cell(&mut b, [32.0, 40.0, -48.0, -40.0], 1.25, [column, row]);
                assert_eq!(a.colors, b.colors);
                assert_eq!(a.indices, b.indices);
                assert_eq!(a.normals, b.normals);
                assert!(a.positions.len() <= 12);
                for (a, b) in a.positions.iter().zip(&b.positions) {
                    assert!(near(a[0] + 32.0, b[0]) && a[1] == b[1] && near(a[2] - 48.0, b[2]));
                }
            }
        }
    }
}
