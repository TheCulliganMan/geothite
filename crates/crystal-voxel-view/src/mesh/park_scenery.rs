//! Source-complete park fixtures; spray and water remain live atlas surfaces.
use super::*;
#[path = "park_scenery_source.rs"]
mod source;
use crate::live_profiles::{Document, Object};
use source::{Identity, Kind};
pub(super) struct Placement {
    resolved: source::Match,
}
impl Placement {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> {
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
fn applies(map: &str, object: &Object) -> bool {
    object.map.as_deref().is_none_or(|m| m == map)
        && object
            .maps
            .as_ref()
            .is_none_or(|maps| maps.iter().any(|m| m == map))
}
fn profile_owns_source(map: &str, s: &VisualTileSource, document: Option<&Document>) -> bool {
    document.is_some_and(|d| {
        d.objects.iter().any(|o| {
            applies(map, o)
                && o.tileset == s.tileset_id.as_ref()
                && o.tiles.iter().enumerate().any(|(y, row)| {
                    row.iter().enumerate().any(|(x, &tile)| {
                        let sx = usize::from(o.origin[0]) + x;
                        let sy = usize::from(o.origin[1]) + y;
                        let block = o
                            .metatiles
                            .as_ref()
                            .and_then(|blocks| blocks.get(sy / 4))
                            .and_then(|row| row.get(sx / 4))
                            .copied()
                            .unwrap_or(o.metatile);
                        s.metatile_id == block
                            && usize::from(s.subtile_column) == sx % 4
                            && usize::from(s.subtile_row) == sy % 4
                            && s.tile_index == tile
                    })
                })
        })
    })
}
pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    if !matches!(
        map,
        "NationalPark" | "NationalParkBugContest" | "SafariZoneBeta"
    ) || g.width.checked_mul(g.height) != Some(cells.len())
        || reserved.len() != cells.len()
    {
        return vec![];
    }
    // Even incomplete custom source drawings and overrides on backing/spray cells
    // take priority. No fallback may reconstruct half of a customized fixture.
    let blocked: Vec<_> = cells
        .iter()
        .enumerate()
        .map(|(i, c)| reserved[i] || profile_owns_source(map, &c.source, profiles))
        .collect();
    let ids: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    source::resolve(map, &ids, g.width, g.height, origin, &blocked)
        .into_iter()
        .map(|resolved| Placement { resolved })
        .collect()
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: &Placement,
    claimed: &mut [bool],
) -> bool {
    let p = p.resolved;
    if claimed.len() != cells.len() {
        return false;
    }
    let ids: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    if !p.coherent(&ids, g.width, g.height) || p.guard_indices(g.width).any(|i| claimed[i]) {
        return false;
    }
    let scale = g.tile_height / SOURCE_TILE_HEIGHT;
    let pond = p.kind == Kind::PondBasin;
    let base = if pond {
        crate::profile::WATER_HEIGHT * scale
    } else {
        0.
    };
    for (local, i) in p.indices(g.width).enumerate() {
        let ground = p.ground(local, g.width);
        append_top(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            base,
            g.uv(ground % g.width, ground / g.width),
        );
    }
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let (bounds, rise) = match p.kind {
        Kind::LitterBin => (
            [
                w + 0.20 * g.tile_width,
                w + 1.80 * g.tile_width,
                n + 0.35 * g.tile_height,
                n + 1.90 * g.tile_height,
            ],
            14.0,
        ),
        Kind::PedestalFountain => (
            [
                w + 1.10 * g.tile_width,
                w + 3.90 * g.tile_width,
                n + 0.20 * g.tile_height,
                n + 1.90 * g.tile_height,
            ],
            13.0,
        ),
        Kind::PondBasin => {
            let cx = w + 4.0 * g.tile_width;
            let cz = n + 2.15 * g.tile_height;
            (
                [
                    cx - 0.88 * g.tile_width,
                    cx + 0.88 * g.tile_width,
                    cz - 0.65 * g.tile_height,
                    cz + 0.65 * g.tile_height,
                ],
                6.0,
            )
        }
    };
    crate::park_scenery_models::model(p.kind.asset_index()).append_fitted(
        &mut mesh.solid,
        bounds,
        base,
        rise * scale,
    );
    if pond {
        // Preserve the existing grouped cap's exact footprint, atlas coordinates
        // and top datum. The source image is refreshed by the real runtime;
        // no animation cadence, source handles or baked pixels enter the kit.
        let cx = w + 4.0 * g.tile_width;
        let cz = n + 2.15 * g.tile_height;
        let uv = |x: f32, z: f32| {
            [
                (x - g.origin_x) / (g.width as f32 * g.tile_width),
                (z - g.origin_z) / (g.height as f32 * g.tile_height),
            ]
        };
        let top = base + 5.0 * scale;
        let mut fan = vec![([cx, top, cz], uv(cx, cz))];
        for segment in 0..=24 {
            let a = -(segment as f32 / 24.0 * std::f32::consts::TAU);
            let x = cx + a.cos() * 0.74 * g.tile_width;
            let z = cz + a.sin() * 0.50 * g.tile_height;
            fan.push(([x, top, z], uv(x, z)));
        }
        append_polygon(&mut mesh.textured, &fan, [0., 1., 0.], TEXTURED_SHADE);
    }
    for i in p.indices(g.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some(p.kind.label());
        }
    }
    true
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    fn fixture(pond: bool) -> (Vec<VisualTile>, GridGeometry) {
        let west = if pond {
            [
                0x14, 0x14, 0x14, 0x5f, 0x14, 0x14, 0x5f, 0x80, 0x14, 0x14, 0x5f, 0x90, 0x14, 0x14,
                0x14, 0x5f,
            ]
        } else {
            [
                0x5a, 0x5b, 0, 0x16, 0x13, 0x82, 6, 0, 0, 0x16, 0, 0x16, 6, 0, 6, 0,
            ]
        };
        let east = [
            0x5f, 0x14, 0x14, 0x14, 0x81, 0x5f, 0x14, 0x14, 0x91, 0x5f, 0x14, 0x14, 0x5f, 0x14,
            0x14, 0x14,
        ];
        let width = if pond { 8 } else { 4 };
        let tiles = (0..width * 4)
            .map(|i| {
                let x = i % width;
                let y = i / width;
                VisualTile {
                    animation_frames: None,
                    column: x as u32,
                    row: y as u32,
                    source: crystal_render_api::VisualTileSource {
                        tileset_id: Arc::from("park"),
                        metatile_id: if pond {
                            if x < 4 { 0x3f } else { 0x33 }
                        } else {
                            0x0f
                        },
                        subtile_column: (x % 4) as u8,
                        subtile_row: y as u8,
                        tile_index: if x < 4 {
                            west[y * 4 + x]
                        } else {
                            east[y * 4 + x - 4]
                        },
                    },
                    texture: Default::default(),
                    priority: false,
                }
            })
            .collect();
        (
            tiles,
            GridGeometry {
                width,
                height: 4,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            },
        )
    }
    #[test]
    fn park_bins_and_complete_live_basins_own_only_the_solid_drawing() {
        for pond in [false, true] {
            let (tiles, g) = fixture(pond);
            let cells: Vec<_> = tiles.iter().collect();
            let mut claimed = vec![false; tiles.len()];
            let placements = resolve("NationalPark", &cells, &g, [0, 0], None, &claimed);
            assert_eq!(placements.len(), 1);
            let mut mesh = TerrainMeshData::default();
            mesh.authored_cells = vec![None; tiles.len()];
            assert!(append(&mut mesh, &cells, &g, &placements[0], &mut claimed));
            assert_eq!(claimed.iter().filter(|&&v| v).count(), 4);
            assert!(mesh.solid.positions.len() > 100);
            assert!(
                mesh.solid
                    .positions
                    .iter()
                    .all(|p| p.iter().all(|v| v.is_finite()))
            );
            if pond {
                // Spray identities remain outside ownership. Water backing is
                // still -2; the live grouped cap is still -2+5=3 source pixels.
                for (i, c) in cells.iter().enumerate() {
                    if matches!(c.source.tile_index, 0x14 | 0x5f) {
                        assert!(!claimed[i]);
                    }
                }
                let ys: std::collections::BTreeSet<_> = mesh
                    .textured
                    .positions
                    .iter()
                    .map(|p| p[1].to_bits())
                    .collect();
                assert_eq!(
                    ys,
                    [(-2.0f32).to_bits(), 3.0f32.to_bits()]
                        .into_iter()
                        .collect()
                );
                assert!(
                    resolve(
                        "SafariZoneBeta",
                        &cells,
                        &g,
                        [0, 0],
                        None,
                        &vec![false; tiles.len()]
                    )
                    .is_empty()
                );
            } else {
                assert!(mesh.textured.positions.iter().all(|p| p[1] == 0.));
            }
            let before = mesh.solid.positions.len();
            assert!(!append(&mut mesh, &cells, &g, &placements[0], &mut claimed));
            assert_eq!(before, mesh.solid.positions.len());
        }
    }
    #[test]
    fn incomplete_custom_park_profiles_and_backing_overrides_take_priority() {
        let (tiles, g) = fixture(false);
        let cells: Vec<_> = tiles.iter().collect();
        // Deliberately no usable ground: raw source-level ownership still wins.
        let custom:Document=serde_json::from_str(r#"{"objects":[{"name":"My one-cell bin","map":"NationalPark","tileset":"park","metatile":15,"origin":[0,0],"tiles":[[90]],"ground":65535,"top_pixels":8,"depth_pixels":9}]}"#).unwrap();
        assert!(
            resolve(
                "NationalPark",
                &cells,
                &g,
                [0, 0],
                Some(&custom),
                &[false; 16]
            )
            .is_empty()
        );
        let mut blocked = [false; 16];
        blocked[15] = true;
        assert!(resolve("NationalPark", &cells, &g, [0, 0], None, &blocked).is_empty());
    }
    #[test]
    fn changed_park_backing_never_appends_partial_geometry() {
        let (mut tiles, g) = fixture(false);
        let p = resolve(
            "NationalPark",
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            [0, 0],
            None,
            &[false; 16],
        );
        tiles[15].source.tile_index = 0x99;
        let mut mesh = TerrainMeshData::default();
        let mut claimed = [false; 16];
        assert!(!append(
            &mut mesh,
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            &p[0],
            &mut claimed
        ));
        assert!(mesh.solid.positions.is_empty() && mesh.textured.positions.is_empty());
    }
}
