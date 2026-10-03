use super::*;
use bevy::render::mesh::VertexAttributeValues;

fn triangle(offset: f32) -> SurfaceMeshData {
    SurfaceMeshData {
        positions: vec![
            [offset, 0.0, 0.0],
            [offset + 1.0, 0.0, 0.0],
            [offset, 1.0, 0.0],
        ],
        normals: vec![[0.0, 0.0, 1.0]; 3],
        uvs: vec![[0.25, 0.5]; 3],
        colors: vec![[1.0; 4]; 3],
        indices: vec![0, 1, 2],
        ..Default::default()
    }
}

#[test]
fn indoor_cutaway_marks_only_selected_vertices_without_changing_geometry() {
    let original = triangle(0.0);
    let mut data = original.clone();
    data.cutaway_ranges.push(0..3);
    let ordinary = original.into_mesh();
    let marked = data.into_mesh();
    assert!(ordinary.attribute(Mesh::ATTRIBUTE_UV_1).is_none());
    assert!(matches!(marked.attribute(Mesh::ATTRIBUTE_UV_1),
        Some(VertexAttributeValues::Float32x2(mask)) if mask == &vec![[1.0, 0.0]; 3]));
    for attribute in [
        Mesh::ATTRIBUTE_POSITION,
        Mesh::ATTRIBUTE_NORMAL,
        Mesh::ATTRIBUTE_UV_0,
        Mesh::ATTRIBUTE_COLOR,
    ] {
        assert_eq!(
            marked.attribute(attribute.id).unwrap().get_bytes(),
            ordinary.attribute(attribute.id).unwrap().get_bytes()
        );
    }
    assert_eq!(
        marked.indices().unwrap().iter().collect::<Vec<_>>(),
        ordinary.indices().unwrap().iter().collect::<Vec<_>>()
    );
}

#[test]
fn indoor_cutaway_ranges_survive_merged_surfaces_without_claiming_neighbors() {
    let mut terrain = TerrainMeshData::default();
    terrain.solid = triangle(0.0);
    terrain.animated_solid = triangle(2.0);
    terrain.animated_solid.cutaway_ranges.push(0..3);
    let (_, solid) = terrain.into_meshes();
    let Some(VertexAttributeValues::Float32x2(mask)) = solid.attribute(Mesh::ATTRIBUTE_UV_1) else {
        panic!("marked surface lost its mask during merge");
    };
    assert_eq!(
        mask,
        &vec![
            [0.0, 0.0],
            [0.0, 0.0],
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 0.0],
            [1.0, 0.0]
        ]
    );
}

#[test]
fn authored_exterior_buildings_reveal_only_their_new_geometry() {
    use crate::{exterior_models, new_bark_models};
    use std::sync::Arc;

    for (map, tileset, blocks, door, connected) in [
        ("NewBarkTown", "johto", &[&[0x14, 0x15][..]][..], 3, true),
        (
            "AzaleaTown",
            "johto",
            &[&[0x18, 0x1f, 0x19][..], &[0x1c, 0x1d, 0x1e][..]][..],
            5,
            false,
        ),
        (
            "GoldenrodCity",
            "johto_modern",
            &[&[0x18, 0x1f, 0x19][..], &[0x1c, 0x1d, 0x1e][..]][..],
            5,
            false,
        ),
    ] {
        let width = blocks[0].len() * 4;
        let height = blocks.len() * 4;
        let g = GridGeometry {
            width,
            height: height + 1,
            tile_width: 12.0,
            tile_height: 8.0,
            origin_x: -24.0,
            origin_z: 16.0,
        };
        let p = BuildingPlacement {
            column: 0,
            row: 0,
            width,
            height,
            roof_rows: height / 2,
            ground_tile_index: 6,
        };
        let mut cells = Vec::new();
        for row in 0..=height {
            for column in 0..width {
                let is_ground = row == height;
                let door_tile =
                    if row >= height - 2 && row < height && (door - 1..=door).contains(&column) {
                        [[0x37, 0x38], [0x39, 0x3a]][row - (height - 2)][column - (door - 1)]
                    } else {
                        0
                    };
                cells.push(VisualTile {
                    animation_frames: None,
                    column: column as u32,
                    row: row as u32,
                    source: VisualTileSource {
                        tileset_id: Arc::from(tileset),
                        metatile_id: if is_ground {
                            1
                        } else {
                            blocks[row / 4][column / 4]
                        },
                        subtile_column: (column % 4) as u8,
                        subtile_row: (row % 4) as u8,
                        tile_index: if is_ground { 6 } else { door_tile },
                    },
                    texture: Handle::default(),
                    priority: false,
                });
            }
        }
        let shapes = vec![CellShape::Flat; cells.len()];
        let mut claimed = vec![false; cells.len()];
        let mut terrain = TerrainMeshData {
            solid: triangle(-500.0),
            footing_heights: vec![7.0; cells.len()],
            authored_cells: vec![None; cells.len()],
            ..Default::default()
        };
        let mut expected = terrain.solid.clone();
        let b = [
            g.origin_x,
            g.origin_x + width as f32 * g.tile_width,
            g.origin_z,
            g.origin_z + height as f32 * g.tile_height,
        ];
        let door_x = Some(g.origin_x + door as f32 * g.tile_width);
        if tileset == "johto_modern" {
            exterior_models::model(exterior_models::Kind::ModernGym).append_fitted(
                &mut expected,
                b,
                0.0,
                g.tile_height * 2.0,
                door_x,
            );
        } else {
            let kind = if connected {
                new_bark_models::ModelKind::House
            } else {
                new_bark_models::ModelKind::VioletGym
            };
            new_bark_models::model(kind).append_fitted(
                &mut expected,
                b,
                0.0,
                g.tile_height * 2.0,
                door_x,
            );
        }
        let refs: Vec<_> = cells.iter().collect();
        let appended = if connected {
            new_bark::append_building(&mut terrain, &refs, &shapes, &g, map, p, &mut claimed)
        } else {
            modeled_exteriors::append_building(
                &mut terrain,
                &refs,
                &shapes,
                &g,
                map,
                p,
                &mut claimed,
            )
        };
        assert!(appended, "{map}");
        let building_end = terrain.solid.positions.len();
        expected.cutaway_ranges.push(3..building_end);
        // Geometry, indices, UV0 and baked colors are identical to the original fit.
        assert_eq!(terrain.solid, expected, "{map}");
        assert!(terrain.textured.cutaway_ranges.is_empty(), "backing: {map}");
        assert_eq!(terrain.footing_heights, vec![7.0; cells.len()]);
        assert!(claimed[..width * height].iter().all(|c| *c));
        assert!(claimed[width * height..].iter().all(|c| !c));

        // A changed source phase rejects the same plot without adding geometry,
        // changing eligibility, or modifying existing source ownership/support.
        cells[0].source.subtile_column = 1;
        let refs: Vec<_> = cells.iter().collect();
        let before = terrain.clone();
        let before_claimed = claimed.clone();
        let appended = if connected {
            new_bark::append_building(&mut terrain, &refs, &shapes, &g, map, p, &mut claimed)
        } else {
            modeled_exteriors::append_building(
                &mut terrain,
                &refs,
                &shapes,
                &g,
                map,
                p,
                &mut claimed,
            )
        };
        assert!(!appended, "{map}");
        assert_eq!(terrain, before);
        assert_eq!(claimed, before_claimed);

        // Later solid props and independently merged animated geometry stay
        // unmarked, even after copying and transforming the terrain surface.
        append_solid_quad(
            &mut terrain.solid,
            [
                [900.0, 0.0, 0.0],
                [900.0, 0.0, 1.0],
                [901.0, 0.0, 1.0],
                [901.0, 0.0, 0.0],
            ],
            [0.0, 1.0, 0.0],
            [1.0; 4],
        );
        terrain.animated_solid = triangle(1000.0);
        let mut copy = terrain.clone();
        for p in &mut copy.solid.positions {
            *p = [p[0] * 1.5 + 32.0, p[1] * 0.5, p[2] * 2.0 - 16.0];
        }
        let (_, solid) = copy.into_meshes();
        let Some(VertexAttributeValues::Float32x2(mask)) = solid.attribute(Mesh::ATTRIBUTE_UV_1)
        else {
            panic!("{map}: exterior building lost player reveal during merge");
        };
        assert!(mask[..3].iter().all(|m| *m == [0.0, 0.0]));
        assert!(mask[3..building_end].iter().all(|m| *m == [1.0, 0.0]));
        assert!(mask[building_end..].iter().all(|m| *m == [0.0, 0.0]));
    }
}
