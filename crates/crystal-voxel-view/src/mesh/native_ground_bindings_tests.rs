use super::*;
use std::sync::Arc;

fn fixture(b: &'static Binding, origin: [i32; 2]) -> (Vec<VisualTile>, GridGeometry) {
    let g = GridGeometry {
        width: 12,
        height: 12,
        tile_width: 8.0,
        tile_height: 8.0,
        origin_x: -48.0,
        origin_z: -48.0,
    };
    let mut cells = Vec::new();
    for y in 0..g.height {
        for x in 0..g.width {
            let wx = x as i32 + origin[0];
            let wy = y as i32 + origin[1];
            let dx = wx - 4 - i32::from(b.origin[0]);
            let dy = wy - 4 - i32::from(b.origin[1]);
            let object = (0..b.size as i32).contains(&dx) && (0..b.size as i32).contains(&dy);
            cells.push(VisualTile {
                animation_frames: None,
                column: x as u32,
                row: y as u32,
                source: VisualTileSource {
                    tileset_id: Arc::from(b.tileset),
                    metatile_id: if object { b.block } else { b.ground_block },
                    subtile_column: wx.rem_euclid(4) as u8,
                    subtile_row: wy.rem_euclid(4) as u8,
                    tile_index: if !object {
                        b.ground_tile
                    } else if b.kind == Kind::Grass {
                        0x04
                    } else {
                        [[0x2a, 0x2b], [0x3a, 0x3b]][dy as usize][dx as usize]
                    },
                },
                texture: Handle::default(),
                priority: false,
            });
        }
    }
    (cells, g)
}
fn placements(
    b: &Binding,
    cells: &[VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    d: Option<&Document>,
) -> Vec<Placement> {
    resolve(
        b.map,
        &cells.iter().collect::<Vec<_>>(),
        g,
        origin,
        d,
        &vec![false; cells.len()],
    )
}
#[test]
fn exact_native_drawings_resolve_in_full_padded_and_scrolled_frames() {
    for b in BINDINGS {
        for origin in [[0, 0], [-3, -1], [1, 2]] {
            let (cells, g) = fixture(b, origin);
            let p = placements(b, &cells, &g, origin, None);
            assert_eq!(p.len(), 1, "{b:?} origin {origin:?}");
            assert_eq!(p[0].indices(g.width).count(), b.size * b.size);
            assert_eq!(cells[p[0].ground].source.tile_index, b.ground_tile);
            assert!(native_cell(&cells[p[0].ground], b, origin));
        }
    }
}
#[test]
fn every_source_identity_mutation_rejects_the_complete_object() {
    for b in BINDINGS {
        let (base, g) = fixture(b, [0, 0]);
        let p = placements(b, &base, &g, [0, 0], None).remove(0);
        for i in p.indices(g.width) {
            for mutation in 0..5 {
                let mut cells = base.clone();
                let s = &mut cells[i].source;
                match mutation {
                    0 => s.tileset_id = Arc::from("foreign"),
                    1 => s.metatile_id ^= 1,
                    2 => s.subtile_column ^= 1,
                    3 => s.subtile_row ^= 1,
                    _ => s.tile_index ^= 1,
                }
                assert!(
                    placements(b, &cells, &g, [0, 0], None).is_empty(),
                    "{b:?} cell {i} mutation {mutation}"
                );
            }
        }
        let mut cropped = g;
        cropped.height = p.row + b.size - 1;
        assert!(
            placements(
                b,
                &base[..cropped.width * cropped.height],
                &cropped,
                [0, 0],
                None
            )
            .is_empty()
        );
    }
}
#[test]
fn absent_foreign_wrong_block_or_rephased_backing_keeps_fallback() {
    for b in BINDINGS {
        let (base, g) = fixture(b, [0, 0]);
        for mutation in 0..5 {
            let mut cells = base.clone();
            for c in &mut cells {
                if c.source.metatile_id != b.ground_block {
                    continue;
                }
                match mutation {
                    0 => c.source.tile_index = 0xffff,
                    1 => c.source.tileset_id = Arc::from("foreign"),
                    2 => c.source.metatile_id = 0xffff,
                    3 => c.source.subtile_column ^= 1,
                    _ => c.source.tile_index = 0x11,
                }
            }
            assert!(
                placements(b, &cells, &g, [0, 0], None).is_empty(),
                "{b:?} mutation {mutation}"
            );
        }
    }
}
#[test]
fn same_atlas_ground_in_connected_map_halo_is_not_native_backing() {
    for b in BINDINGS {
        let origin = [-4, -4];
        let (mut cells, g) = fixture(b, origin);
        for c in &mut cells {
            if c.source.metatile_id == b.ground_block && native_cell(c, b, origin) {
                c.source.tile_index = 0xffff;
            }
        }
        assert!(cells.iter().any(|c| c.source.tile_index == b.ground_tile));
        assert!(placements(b, &cells, &g, origin, None).is_empty());
    }
}
#[test]
fn map_scope_and_available_native_lawn_preserve_original_adapter() {
    for b in BINDINGS {
        let (mut cells, g) = fixture(b, [0, 0]);
        let refs = cells.iter().collect::<Vec<_>>();
        for map in ["Route17", "NewBarkTown", "Route40", "CustomMap"] {
            assert!(resolve(map, &refs, &g, [0, 0], None, &vec![false; cells.len()]).is_empty());
        }
        cells[0].source.metatile_id = if b.kind == Kind::Grass { 0x02 } else { 0x01 };
        cells[0].source.tile_index = b.preferred_ground;
        assert!(placements(b, &cells, &g, [0, 0], None).is_empty());
    }
}
fn profile(b: &Binding, map: Option<&str>, origin: [u8; 2], tile: u16, block: u16) -> Document {
    Document {
        objects: vec![crate::live_profiles::Object {
            name: "Custom source binding".into(),
            tileset: b.tileset.into(),
            map: map.map(str::to_owned),
            maps: None,
            metatile: block,
            metatiles: None,
            origin,
            tiles: vec![vec![tile, 999], vec![998, 997]],
            ground: 999,
            top_pixels: 0,
            depth_pixels: 2.0,
            footing_pixels: Some(17.0),
            mask: Default::default(),
            parts: Vec::new(),
        }],
        ..Default::default()
    }
}
#[test]
fn partial_custom_objects_and_custom_backing_get_first_refusal() {
    for b in BINDINGS {
        let (cells, g) = fixture(b, [0, 0]);
        let tile = if b.kind == Kind::Grass { 0x04 } else { 0x2a };
        let d = profile(b, Some(b.map), b.origin, tile, b.block);
        assert!(placements(b, &cells, &g, [0, 0], Some(&d)).is_empty());
        let unrelated = profile(b, Some("AnotherMap"), b.origin, tile, b.block);
        assert_eq!(placements(b, &cells, &g, [0, 0], Some(&unrelated)).len(), 1);
        // All phases of the only backing source belong to a custom floor.
        let mut floor = profile(b, None, [0, 0], b.ground_tile, b.ground_block);
        floor.objects[0].tiles = vec![vec![b.ground_tile; 4]; 4];
        assert!(placements(b, &cells, &g, [0, 0], Some(&floor)).is_empty());
    }
}
#[test]
fn existing_ownership_rejects_whole_objects_without_partial_writes() {
    for b in BINDINGS {
        let (cells, g) = fixture(b, [0, 0]);
        let refs = cells.iter().collect::<Vec<_>>();
        let p = placements(b, &cells, &g, [0, 0], None).remove(0);
        let mut claimed = vec![false; cells.len()];
        claimed[p.indices(g.width).last().unwrap()] = true;
        assert!(resolve(b.map, &refs, &g, [0, 0], None, &claimed).is_empty());
        let mut mesh = TerrainMeshData::default();
        assert!(!append(&mut mesh, &refs, &g, &p, &mut claimed));
        assert!(mesh.solid.positions.is_empty() && mesh.textured.positions.is_empty());
    }
}
#[test]
fn append_preserves_sources_footing_native_uv_seams_and_prevents_duplicates() {
    for b in BINDINGS {
        let (cells, g) = fixture(b, [0, 0]);
        let refs = cells.iter().collect::<Vec<_>>();
        let before = cells.iter().map(|c| c.source.clone()).collect::<Vec<_>>();
        let p = placements(b, &cells, &g, [0, 0], None).remove(0);
        let count = b.size * b.size;
        let mut mesh = TerrainMeshData {
            authored_cells: vec![None; cells.len()],
            footing_heights: vec![3.25; cells.len()],
            ..Default::default()
        };
        let mut claimed = vec![false; cells.len()];
        assert!(append(&mut mesh, &refs, &g, &p, &mut claimed));
        assert_eq!(mesh.footing_heights, vec![3.25; cells.len()]);
        assert_eq!(
            before,
            cells.iter().map(|c| c.source.clone()).collect::<Vec<_>>()
        );
        assert_eq!(claimed.iter().filter(|&&v| v).count(), count);
        assert_eq!(
            mesh.authored_cells.iter().filter(|v| v.is_some()).count(),
            count
        );
        assert_eq!(mesh.textured.indices.len(), count * 6);
        assert!(
            mesh.textured
                .positions
                .iter()
                .all(|v| v[1] == b.backing_height(g.tile_height))
        );
        assert!(!mesh.solid.indices.is_empty());
        assert!(mesh.solid.positions.iter().all(|v| v[1] >= -0.001));
        let (u0, u1, v0, v1) = g.uv(p.ground % g.width, p.ground / g.width);
        assert!(
            mesh.textured
                .uvs
                .iter()
                .all(|uv| uv[0] >= u0 && uv[0] <= u1 && uv[1] >= v0 && uv[1] <= v1)
        );
        let old = (mesh.textured.indices.len(), mesh.solid.indices.len());
        assert!(!append(&mut mesh, &refs, &g, &p, &mut claimed));
        assert_eq!(old, (mesh.textured.indices.len(), mesh.solid.indices.len()));
    }
}
#[test]
fn changed_source_or_backing_after_resolve_cannot_append() {
    for b in BINDINGS {
        let (base, g) = fixture(b, [0, 0]);
        let p = placements(b, &base, &g, [0, 0], None).remove(0);
        for i in [p.ground, p.indices(g.width).last().unwrap()] {
            let mut cells = base.clone();
            cells[i].source.tile_index = 0xffff;
            let mut mesh = TerrainMeshData::default();
            let mut claimed = vec![false; cells.len()];
            assert!(!append(
                &mut mesh,
                &cells.iter().collect::<Vec<_>>(),
                &g,
                &p,
                &mut claimed
            ));
            assert!(mesh.textured.positions.is_empty() && mesh.solid.positions.is_empty());
            assert!(!claimed.iter().any(|&v| v));
        }
    }
}

#[test]
fn rock_binding_reuses_existing_geometry_and_palette_at_both_tile_scales() {
    for b in BINDINGS.iter().filter(|b| b.kind == Kind::LandRock) {
        for tile_height in [8.0, 16.0] {
            let (cells, mut g) = fixture(b, [0, 0]);
            g.tile_height = tile_height;
            let p = placements(b, &cells, &g, [0, 0], None).remove(0);
            let mut mesh = TerrainMeshData {
                authored_cells: vec![None; cells.len()],
                ..Default::default()
            };
            assert!(append(
                &mut mesh,
                &cells.iter().collect::<Vec<_>>(),
                &g,
                &p,
                &mut vec![false; cells.len()]
            ));
            // Supply the old adapter its required lawn on an otherwise identical
            // drawing. Only the floor sample changes; mesh and material must not.
            let mut baseline = cells.clone();
            for cell in &mut baseline {
                if cell.source.metatile_id == b.ground_block {
                    cell.source.metatile_id = 0x01;
                    cell.source.tile_index = 0x2c;
                }
            }
            let refs = baseline.iter().collect::<Vec<_>>();
            let shapes = refs
                .iter()
                .map(|c| shape_for_source_on_map(b.map, &c.source))
                .collect::<Vec<_>>();
            let mut expected = TerrainMeshData::default();
            modeled_exteriors::append_props(
                &mut expected,
                &refs,
                &shapes,
                &g,
                &mut vec![false; cells.len()],
            );
            assert_eq!(mesh.solid, expected.solid, "{b:?} scale {tile_height}");
        }
    }
}
