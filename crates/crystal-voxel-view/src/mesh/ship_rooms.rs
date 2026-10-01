//! Source-complete Fast Ship furnishings and joined lower-deck bulkheads.
//! Sparse architecture claims never fill the room, and all floors/footing keep
//! native support. Source identities, complete joins and custom live ownership
//! are proved before either suppressing source art or emitting an original model.
use super::*;
use crate::interior_models::Model;
use crate::live_profiles::Document;
use std::sync::OnceLock;
#[path = "ship_room_sources.rs"]
mod sources;
use sources::Asset;

impl Asset {
    fn model(self) -> &'static Model {
        if self == Self::WallTea {
            return Self::TeaShort.model();
        }
        static MODELS: OnceLock<[Model; 7]> = OnceLock::new();
        &MODELS.get_or_init(|| {
            [
                include_str!("../../models/ship_rooms/tea_table_short.mesh.json"),
                include_str!("../../models/lighthouse_chamber/tea_table.mesh.json"),
                include_str!("../../models/ship_rooms/tea_table_long.mesh.json"),
                include_str!("../../models/ship_rooms/tea_table_mess.mesh.json"),
                include_str!("../../models/ship_rooms/captains_desk.mesh.json"),
                include_str!("../../models/ship_rooms/lower_bulkhead_u.mesh.json"),
                include_str!("../../models/ship_rooms/captains_chair.mesh.json"),
            ]
            .map(|s| Model::parse(s).expect("validated original ship room asset"))
        })[if self == Self::CaptainsChair {
            6
        } else {
            self as usize
        }]
    }
    fn label(self) -> &'static str {
        match self {
            Self::TeaShort => "ship/tea-table-short",
            Self::TeaMedium => "ship/tea-table-medium",
            Self::TeaLong => "ship/tea-table-long",
            Self::TeaMess => "ship/tea-table-mess",
            Self::CaptainsDesk => "ship/captains-logbook-desk",
            Self::BulkheadU => "ship/lower-deck-bulkhead",
            Self::WallTea => "ship/wall-side-cup-table",
            Self::CaptainsChair => "ship/captains-highback-chair",
            Self::StoolEast | Self::StoolWest => "dungeon:ship-stool",
            Self::FullBunk | Self::JoinedBunk => "dungeon:ship-bunk",
        }
    }
}
pub(super) struct Placement {
    asset: Asset,
    column: usize,
    row: usize,
    ground: [usize; 2],
}
impl Placement {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        let (w, h, _) = self.asset.size();
        (0..h).flat_map(move |y| {
            (0..w).filter_map(move |x| {
                self.asset
                    .claim(x, y)
                    .then_some((self.row + y) * width + self.column + x)
            })
        })
    }
}
fn valid_ground(cell: &VisualTile, parity: usize) -> bool {
    let s = &cell.source;
    s.tileset_id.as_ref() == "lighthouse"
        && s.metatile_id == 0x0b
        && s.subtile_column < 4
        && s.subtile_row < 4
        && usize::from((s.subtile_column + s.subtile_row) % 2) == parity
        && s.tile_index == if parity == 0 { 0x0d } else { 0x1d }
}
fn complete(cells: &[&VisualTile], g: &GridGeometry, p: &Placement) -> bool {
    sources::complete(p.asset, g.width, g.height, p.column, p.row, |x, y| {
        cells.get(y * g.width + x).and_then(|c| {
            let s = &c.source;
            (s.tileset_id.as_ref() == "lighthouse").then_some((
                s.metatile_id,
                s.subtile_column,
                s.subtile_row,
                s.tile_index,
            ))
        })
    })
}
pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    if !sources::applies(map) || cells.len() != g.width * g.height || reserved.len() != cells.len()
    {
        return Vec::new();
    }
    let Some(even) = cells.iter().position(|c| valid_ground(c, 0)) else {
        return Vec::new();
    };
    let Some(odd) = cells.iter().position(|c| valid_ground(c, 1)) else {
        return Vec::new();
    };
    let mut blocked = reserved.to_vec();
    // Bundled source wall profiles do not overlap these drawings. Every live
    // profile, including a user's edited or added profile, wins an overlap.
    for p in live::resolve(cells, g.width, g.height, map, profiles) {
        for i in p.indices(g.width) {
            blocked[i] = true;
        }
    }
    let mut out = Vec::new();
    for asset in Asset::ALL {
        if asset == Asset::BulkheadU && map != "FastShipB1F" {
            continue;
        }
        if matches!(asset, Asset::CaptainsDesk | Asset::CaptainsChair)
            && map != "FastShipCabins_SE_SSE_CaptainsCabin"
        {
            continue;
        }
        for row in 0..g.height {
            for column in 0..g.width {
                let p = Placement {
                    asset,
                    column,
                    row,
                    ground: [even, odd],
                };
                if complete(cells, g, &p) && !p.indices(g.width).any(|i| blocked[i]) {
                    for i in p.indices(g.width) {
                        blocked[i] = true;
                    }
                    out.push(p);
                }
            }
        }
    }
    out
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: &Placement,
    claimed: &mut [bool],
) -> bool {
    if claimed.len() != cells.len()
        || cells.len() != g.width * g.height
        || !p
            .ground
            .iter()
            .enumerate()
            .all(|(j, &i)| cells.get(i).is_some_and(|c| valid_ground(c, j)))
        || !complete(cells, g, p)
        || p.indices(g.width).any(|i| claimed[i])
    {
        return false;
    }
    for i in p.indices(g.width) {
        let source = &cells[i].source;
        let parity = usize::from((source.subtile_column + source.subtile_row) % 2);
        let sample = p.ground[parity];
        append_top(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            0.,
            g.uv(sample % g.width, sample / g.width),
        );
    }
    let (width, height, rise) = p.asset.size();
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let mut bounds = [
        w,
        w + width as f32 * g.tile_width,
        n,
        n + height as f32 * g.tile_height,
    ];
    if matches!(p.asset, Asset::StoolEast | Asset::StoolWest) {
        // A round 12px seat stays north of the unchanged native foot anchor
        // at local (8,16), clearing stationary and spinning operator rigs.
        bounds = [w + 2. * g.tile_width / 8., w + 14. * g.tile_width / 8.,
            n, n + 12. * g.tile_height / 8.];
    } else if !matches!(p.asset, Asset::BulkheadU | Asset::FullBunk | Asset::JoinedBunk) {
        bounds[0] += 1.1 * g.tile_width / 8.;
        bounds[1] -= 1.1 * g.tile_width / 8.;
        bounds[2] += 1.3 * g.tile_height / 8.;
        bounds[3] -= 1.3 * g.tile_height / 8.;
    }
    let start = mesh.solid.positions.len();
    match p.asset {
        Asset::StoolEast | Asset::StoolWest | Asset::FullBunk | Asset::JoinedBunk => {
            let kind = if matches!(p.asset, Asset::FullBunk | Asset::JoinedBunk) {
                crate::dungeon_models::Kind::ShipBunk
            } else {
                crate::dungeon_models::Kind::ShipStool
            };
            // Reuse the canonical cache and original editable geometry. The
            // full berth fits its complete 16x32 plot at the same 7px rise.
            crate::dungeon_models::model(kind)
                .append(&mut mesh.solid, bounds, 0., rise * g.tile_height / 8.);
        }
        _ => p.asset.model()
            .append_fitted(&mut mesh.solid, bounds, 0., rise * g.tile_height / 8.),
    }
    // Only wall vertices participate in the local player reveal; floor and
    // furniture stay opaque. This does not alter collision or support height.
    if p.asset == Asset::BulkheadU && mesh.solid.positions.len() > start {
        mesh.solid
            .cutaway_ranges
            .push(start..mesh.solid.positions.len());
    }
    for i in p.indices(g.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some(p.asset.label());
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    fn fixture(asset: Asset) -> (Vec<VisualTile>, GridGeometry) {
        let (w, h, _) = asset.size();
        let g = GridGeometry {
            width: w + 8,
            height: h + 8,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -7.,
            origin_z: 13.,
        };
        let mut cells: Vec<_> = (0..g.width * g.height)
            .map(|i| VisualTile {
                column: (i % g.width) as u32,
                row: (i / g.width) as u32,
                texture: Default::default(),
                priority: false,
                source: VisualTileSource {
                    tileset_id: Arc::from("lighthouse"),
                    metatile_id: 0x0b,
                    subtile_column: (i % g.width % 4) as u8,
                    subtile_row: (i / g.width % 4) as u8,
                    tile_index: if (i % g.width + i / g.width) % 2 == 0 {
                        0x0d
                    } else {
                        0x1d
                    },
                },
            })
            .collect();
        for y in 0..h {
            for x in 0..w {
                if let Some((b, sx, sy, t)) = asset.expected(x, y) {
                    cells[(y + 4) * g.width + x + 4].source = VisualTileSource {
                        tileset_id: Arc::from("lighthouse"),
                        metatile_id: b,
                        subtile_column: sx,
                        subtile_row: sy,
                        tile_index: t,
                    };
                }
            }
        }
        (cells, g)
    }
    #[test]
    fn ship_rooms_reject_source_mutation_claim_conflicts_and_missing_ground() {
        for asset in Asset::ALL {
            let (cells, g) = fixture(asset);
            let map = if asset == Asset::BulkheadU {
                "FastShipB1F"
            } else {
                "FastShipCabins_SE_SSE_CaptainsCabin"
            };
            let refs: Vec<_> = cells.iter().collect();
            let reserved = vec![false; cells.len()];
            let placements = resolve(map, &refs, &g, None, &reserved);
            assert_eq!(placements.len(), 1);
            let p = &placements[0];
            for i in p.indices(g.width) {
                let mut r = reserved.clone();
                r[i] = true;
                assert!(resolve(map, &refs, &g, None, &r).is_empty());
            }
            let (w, h, _) = asset.size();
            for y in 0..h {
                for x in 0..w {
                    if asset.expected(x, y).is_none() {
                        continue;
                    }
                    let mut bad = cells.clone();
                    bad[(y + 4) * g.width + x + 4].source.tile_index ^= 1;
                    assert!(
                        resolve(map, &bad.iter().collect::<Vec<_>>(), &g, None, &reserved)
                            .is_empty()
                    );
                }
            }
            for phase in 0..2 {
                let mut bad = cells.clone();
                for c in &mut bad {
                    if valid_ground(c, phase) {
                        c.source.tile_index = 0xff;
                    }
                }
                assert!(
                    resolve(map, &bad.iter().collect::<Vec<_>>(), &g, None, &reserved).is_empty()
                );
            }
            assert!(resolve("OlivineLighthouse6F", &refs, &g, None, &reserved).is_empty());
        }
    }
    #[test]
    fn ship_rooms_append_revalidates_source_without_partial_mutation() {
        for asset in Asset::ALL {
            let (mut cells, g) = fixture(asset);
            let p = Placement {
                asset,
                column: 4,
                row: 4,
                ground: [0, 1],
            };
            cells[4 * g.width + 4].source.tile_index ^= 1;
            let refs: Vec<_> = cells.iter().collect();
            let mut mesh = TerrainMeshData::default();
            mesh.authored_cells = vec![None; cells.len()];
            mesh.footing_heights = vec![1.25; cells.len()];
            let before = mesh.clone();
            let mut claimed = vec![false; cells.len()];
            assert!(!append(&mut mesh, &refs, &g, &p, &mut claimed));
            assert_eq!(mesh, before);
            assert!(claimed.iter().all(|v| !*v));
        }
    }
    #[test]
    fn ship_rooms_append_preserves_footing_and_sparse_floor_exclusion() {
        for asset in Asset::ALL {
            let (cells, g) = fixture(asset);
            let refs: Vec<_> = cells.iter().collect();
            let p = Placement {
                asset,
                column: 4,
                row: 4,
                ground: [0, 1],
            };
            let mut mesh = TerrainMeshData::default();
            mesh.authored_cells = vec![None; cells.len()];
            mesh.footing_heights = vec![2.25; cells.len()];
            let footing = mesh.footing_heights.clone();
            let mut claimed = vec![false; cells.len()];
            assert!(append(&mut mesh, &refs, &g, &p, &mut claimed));
            assert_eq!(mesh.footing_heights, footing);
            assert_eq!(
                claimed.iter().filter(|&&v| v).count(),
                p.indices(g.width).count()
            );
            assert!(mesh.textured.positions.iter().all(|p| p[1] == 0.));
            assert_eq!(
                mesh.solid.cutaway_ranges.is_empty(),
                asset != Asset::BulkheadU
            );
            if asset == Asset::BulkheadU {
                for y in 0..14 {
                    for x in 2..22 {
                        assert!(!claimed[(4 + y) * g.width + 4 + x]);
                    }
                }
                let (w, _, n, _) = g.bounds(4, 4);
                assert!(
                    mesh.solid
                        .positions
                        .iter()
                        .all(|v| sources::bulkhead_contains(v[0] - w, v[2] - n))
                );
            }
            let before = mesh.clone();
            assert!(!append(&mut mesh, &refs, &g, &p, &mut claimed));
            assert_eq!(mesh, before);
        }
    }
    #[test]
    fn ship_rooms_round_stools_and_full_bunks_reuse_original_cached_geometry() {
        for asset in [Asset::StoolEast, Asset::StoolWest, Asset::FullBunk, Asset::JoinedBunk] {
            let (cells, g) = fixture(asset);
            let refs: Vec<_> = cells.iter().collect();
            let p = Placement { asset, column: 4, row: 4, ground: [0, 1] };
            let mut mesh = TerrainMeshData::default();
            let mut claimed = vec![false; cells.len()];
            assert!(append(&mut mesh, &refs, &g, &p, &mut claimed));
            let (w, _, n, _) = g.bounds(4, 4);
            let (kind, bounds) = if matches!(asset, Asset::FullBunk | Asset::JoinedBunk) {
                (crate::dungeon_models::Kind::ShipBunk, [w, w + 16., n, n + 32.])
            } else {
                (crate::dungeon_models::Kind::ShipStool, [w + 2., w + 14., n, n + 12.])
            };
            let mut cached = SurfaceMeshData::default();
            crate::dungeon_models::model(kind).append(&mut cached, bounds, 0., 7.);
            assert_eq!(mesh.solid, cached, "reuse must use the canonical mesh/cache/fitting");
            assert_eq!(claimed.iter().filter(|&&v| v).count(), if matches!(asset, Asset::FullBunk | Asset::JoinedBunk) { 8 } else { 4 });
        }
    }
    #[test]
    fn ship_rooms_new_furniture_yields_to_every_custom_profile_and_rejects_stale_feet() {
        for asset in [Asset::StoolEast, Asset::StoolWest, Asset::FullBunk, Asset::JoinedBunk] {
            let (mut cells, g) = fixture(asset);
            let (w, h, _) = asset.size();
            let (block, x, y, _) = asset.expected(0, 0).unwrap();
            let tiles: Vec<Vec<_>> = (0..h).map(|dy| (0..w)
                .map(|dx| asset.expected(dx, dy).unwrap().3).collect()).collect();
            let custom: Document = serde_json::from_value(serde_json::json!({"objects":[{
                "name":"My source-complete ship furniture", "map":"FastShipB1F",
                "tileset":"lighthouse", "metatile":block, "origin":[x,y], "tiles":tiles,
                "metatiles": if asset == Asset::JoinedBunk { Some(vec![vec![0x38], vec![0x39]]) } else { None },
                "ground":13, "top_pixels":8, "depth_pixels":12
            }]})).unwrap();
            let empty = vec![false; cells.len()];
            assert!(resolve("FastShipB1F", &cells.iter().collect::<Vec<_>>(), &g,
                Some(&custom), &empty).is_empty());
            if asset == Asset::JoinedBunk {
                for start in [0, 2] {
                    // A custom upper or lower half owns the entire compound
                    // model, even when its other half has unchanged source art.
                    let mut half = custom.clone();
                    let (block, x, y, _) = asset.expected(0, start).unwrap();
                    half.objects[0].metatiles = None;
                    half.objects[0].metatile = block;
                    half.objects[0].origin = [x, y];
                    half.objects[0].tiles = (start..start + 2).map(|dy| (0..2)
                        .map(|dx| asset.expected(dx, dy).unwrap().3).collect()).collect();
                    assert!(resolve("FastShipB1F", &cells.iter().collect::<Vec<_>>(), &g,
                        Some(&half), &empty).is_empty());
                }
            }
            let out = resolve("FastShipB1F", &cells.iter().collect::<Vec<_>>(), &g, None, &empty);
            assert_eq!(out.len(), 1);
            for y in 0..h {
                for x in 0..w {
                    let i = (4 + y) * g.width + 4 + x;
                    cells[i].source.tile_index ^= 1;
                    let mut mesh = TerrainMeshData::default();
                    let mut claimed = empty.clone();
                    assert!(!append(&mut mesh, &cells.iter().collect::<Vec<_>>(), &g, &out[0], &mut claimed));
                    assert_eq!(mesh, TerrainMeshData::default());
                    assert_eq!(claimed, empty);
                    cells[i].source.tile_index ^= 1;
                }
            }
        }
    }

}
