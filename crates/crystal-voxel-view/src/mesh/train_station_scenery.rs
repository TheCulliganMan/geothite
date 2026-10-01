//! Source-aware, presentation-only stationary train body. Both native boarding
//! door drawings, all rail backing and the entire gate/platform aisle survive.
use super::*;
#[path = "train_station_source.rs"]
mod source;
use crate::live_profiles::Document;
use source::Identity;
pub(super) struct Placement {
    resolved: source::Match,
}
impl Placement {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        self.resolved.indices(width)
    }
}
fn identity(s: &crystal_render_api::VisualTileSource) -> Identity<'_> {
    Identity {
        tileset: &s.tileset_id,
        metatile: s.metatile_id,
        column: s.subtile_column,
        row: s.subtile_row,
        tile: s.tile_index,
    }
}
pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    if !source::is_map(map)
        || g.width.checked_mul(g.height) != Some(cells.len())
        || cells.len() != reserved.len()
    {
        return Vec::new();
    }
    let mut blocked = reserved.to_vec();
    // No canonical override exists for this body. All custom objects,
    // including ones in either doorway or adjoining platform, win atomically.
    for p in live::resolve(cells, g.width, g.height, map, profiles) {
        for i in p.indices(g.width) {
            blocked[i] = true;
        }
    }
    let identities: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    source::resolve(map, &identities, g.width, g.height, origin, &blocked)
        .into_iter()
        .map(|resolved| Placement { resolved })
        .collect()
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    placement: &Placement,
    claimed: &mut [bool],
) -> bool {
    let p = &placement.resolved;
    let identities: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    if claimed.len() != cells.len()
        || !p.coherent(&identities, g.width, g.height)
        || claimed[p.ground]
        || p.guard_indices(g.width).any(|i| claimed[i])
    {
        return false;
    }
    let uv = g.uv(p.ground % g.width, p.ground / g.width);
    for i in p.indices(g.width) {
        append_top(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            0.,
            uv,
        );
    }
    let (west, _, north, _) = g.bounds(p.column, p.row);
    crate::train_station_models::train().append_fitted(
        &mut mesh.solid,
        [
            west,
            west + 20. * g.tile_width,
            north,
            north + 4. * g.tile_height,
        ],
        0.,
        26.2 * g.tile_height / SOURCE_TILE_HEIGHT,
    );
    for i in p.indices(g.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some("train:stationary-body");
        }
    }
    // The unclaimed door/rail/platform cells use the ordinary live source
    // pass. No footing height, collision, actor, warp or controller changes.
    true
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    fn fixture() -> (Vec<VisualTile>, GridGeometry) {
        let mut tiles = (0..32 * 20)
            .map(|i| VisualTile {
                column: (i % 32) as u32,
                row: (i / 32) as u32,
                source: crystal_render_api::VisualTileSource {
                    tileset_id: Arc::from("train_station"),
                    metatile_id: 0x03,
                    subtile_column: (i % 4) as u8,
                    subtile_row: ((i / 32) % 4) as u8,
                    tile_index: 0x3d,
                },
                texture: Default::default(),
                priority: false,
            })
            .collect::<Vec<_>>();
        // Small source-identity regression fixture, never source pixels.
        let blocks = [0x10, 0x11, 0x15, 0x12, 0x13];
        let drawing = [
            [
                0x1f, 0x05, 0x06, 0x0a, 0x0a, 0x0a, 0x09, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x09,
                0x0a, 0x0a, 0x0a, 0x0b, 0x0c, 0x1f,
            ],
            [
                0x14, 0x15, 0x16, 0x1a, 0x1a, 0x1a, 0x19, 0x1a, 0x1a, 0x1a, 0x1a, 0x1a, 0x1a, 0x19,
                0x1a, 0x1a, 0x1a, 0x1b, 0x1c, 0x1d,
            ],
            [
                0x24, 0x25, 0x26, 0x27, 0x07, 0x2f, 0x29, 0x27, 0x28, 0x28, 0x28, 0x28, 0x2a, 0x29,
                0x07, 0x2f, 0x2a, 0x2b, 0x2c, 0x2d,
            ],
            [
                0x20, 0x1f, 0x2e, 0x1f, 0x17, 0x00, 0x2e, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x2e,
                0x17, 0x00, 0x1f, 0x2e, 0x1f, 0x0f,
            ],
        ];
        for y in 0..8 {
            for x in 0..20 {
                let source = &mut tiles[(y + 8) * 32 + x + 8].source;
                source.metatile_id = if y < 4 { blocks[x / 4] } else { 0x04 };
                source.tile_index = if y < 4 {
                    drawing[y][x]
                } else {
                    [0x32, 0x33, 0x3e, 0x3e][y - 4]
                };
            }
        }
        tiles[8 * 32].source.metatile_id = 0x14;
        tiles[8 * 32].source.tile_index = 0x1f;
        let g = GridGeometry {
            width: 32,
            height: 20,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        (tiles, g)
    }
    #[test]
    fn train_model_has_two_completely_empty_boarding_footprints() {
        let mut surface = SurfaceMeshData::default();
        crate::train_station_models::train().append_fitted(
            &mut surface,
            [0., 160., 0., 32.],
            0.,
            26.2,
        );
        assert!(!surface.indices.is_empty());
        for tri in surface.indices.chunks_exact(3) {
            let vs = tri
                .iter()
                .map(|&i| surface.positions[i as usize])
                .collect::<Vec<_>>();
            for (left, right) in [(32., 48.), (112., 128.)] {
                assert!(
                    vs.iter().all(|v| v[0] <= left + 0.0001)
                        || vs.iter().all(|v| v[0] >= right - 0.0001)
                        || vs.iter().all(|v| v[2] <= 16.0001),
                    "solid covers a native train door"
                );
            }
        }
    }
    #[test]
    fn train_appends_body_once_and_preserves_live_doors_rail_and_footing() {
        let (tiles, g) = fixture();
        let cells = tiles.iter().collect::<Vec<_>>();
        let mut claimed = vec![false; cells.len()];
        let p = resolve(
            "GoldenrodMagnetTrainStation",
            &cells,
            &g,
            [0, 0],
            None,
            &claimed,
        );
        assert_eq!(p.len(), 1);
        let mut mesh = TerrainMeshData::default();
        mesh.authored_cells = vec![None; cells.len()];
        mesh.footing_heights = vec![2.5; cells.len()];
        assert!(append(&mut mesh, &cells, &g, &p[0], &mut claimed));
        assert_eq!(claimed.iter().filter(|&&v| v).count(), 60);
        assert_eq!(
            mesh.authored_cells.iter().filter(|v| v.is_some()).count(),
            60
        );
        assert_eq!(mesh.textured.positions.len(), 60 * 4);
        assert!(mesh.textured.positions.iter().all(|v| v[1] == 0.));
        assert!(mesh.footing_heights.iter().all(|&v| v == 2.5));
        assert!(mesh.solid.cutaway_ranges.is_empty());
        for y in 10..12 {
            for x in [12, 13, 22, 23] {
                assert!(!claimed[y * 32 + x]);
            }
        }
        assert!((12..16).all(|y| (8..28).all(|x| !claimed[y * 32 + x])));
        let before = mesh.solid.positions.len();
        assert!(!append(&mut mesh, &cells, &g, &p[0], &mut claimed));
        assert_eq!(mesh.solid.positions.len(), before);
    }
    #[test]
    fn every_custom_door_platform_or_body_override_rejects_whole_train() {
        let (tiles, g) = fixture();
        let cells = tiles.iter().collect::<Vec<_>>();
        let reserved = vec![false; cells.len()];
        for (block, origin, tile, ground) in [
            (0x10, [1, 0], 0x05, 0x1f),
            (0x11, [0, 2], 0x07, 0x1f),
            (0x04, [0, 2], 0x3e, 0x3d),
        ] {
            let custom:Document=serde_json::from_value(serde_json::json!({"objects":[{
                "name":"Custom station override","tileset":"train_station","map":"GoldenrodMagnetTrainStation",
                "metatile":block,"origin":origin,"tiles":[[tile]],"ground":ground,"top_pixels":1,"depth_pixels":3.0
            }],"atmosphere":null})).unwrap();
            assert!(
                !live::resolve(
                    &cells,
                    g.width,
                    g.height,
                    "GoldenrodMagnetTrainStation",
                    Some(&custom)
                )
                .is_empty()
            );
            assert!(
                resolve(
                    "GoldenrodMagnetTrainStation",
                    &cells,
                    &g,
                    [0, 0],
                    Some(&custom),
                    &reserved
                )
                .is_empty()
            );
        }
    }
    #[test]
    fn stale_or_custom_opening_rejects_before_any_mesh_mutation() {
        let (mut tiles, g) = fixture();
        let p = {
            let cells = tiles.iter().collect::<Vec<_>>();
            resolve(
                "SaffronMagnetTrainStation",
                &cells,
                &g,
                [0, 0],
                None,
                &vec![false; cells.len()],
            )
        };
        let door = 10 * 32 + 12;
        let mut claimed = vec![false; tiles.len()];
        claimed[door] = true;
        let mut mesh = TerrainMeshData::default();
        assert!(!append(
            &mut mesh,
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            &p[0],
            &mut claimed
        ));
        claimed[door] = false;
        tiles[door].source.tile_index ^= 1;
        assert!(!append(
            &mut mesh,
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            &p[0],
            &mut claimed
        ));
        assert!(mesh.textured.positions.is_empty() && mesh.solid.positions.is_empty());
    }
    #[test]
    fn unrelated_shared_atlas_or_incomplete_train_never_resolves() {
        let (mut tiles, g) = fixture();
        let reserved = vec![false; tiles.len()];
        assert!(
            resolve(
                "ViridianGym",
                &tiles.iter().collect::<Vec<_>>(),
                &g,
                [0, 0],
                None,
                &reserved
            )
            .is_empty()
        );
        tiles[8 * 32 + 9].source.tile_index ^= 1;
        assert!(
            resolve(
                "GoldenrodMagnetTrainStation",
                &tiles.iter().collect::<Vec<_>>(),
                &g,
                [0, 0],
                None,
                &reserved
            )
            .is_empty()
        );
    }
}
