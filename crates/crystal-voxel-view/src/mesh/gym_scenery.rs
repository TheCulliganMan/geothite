//! Exact Gym drawings with original planter, broadleaf and connected-wall art.
use super::*;
#[path = "gym_scenery_source.rs"]
mod source;
use crate::live_profiles::Document;
use source::{Identity, Kind};
pub(super) struct Placement {
    resolved: source::Match,
}
impl Placement {
    /// The surface pass may finish only the actual successful model's native
    /// backing sample, after the same complete source drawing resolves again.
    pub(super) fn floor_sample_and_label(&self) -> (usize, &'static str) {
        (self.resolved.ground, self.resolved.kind.label())
    }
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
    if !matches!(
        map,
        "AzaleaGym" | "GoldenrodGym" | "CeladonGym" | "ViridianGym"
    ) || g.width.checked_mul(g.height) != Some(cells.len())
        || reserved.len() != cells.len()
    {
        return Vec::new();
    }
    let mut blocked = reserved.to_vec();
    // Gym scenery has no bundled profile override. Every resolving profile is
    // authoritative, including renamed/custom profiles with the same source art.
    for p in live::resolve(cells, g.width, g.height, map, profiles) {
        for i in p.indices(g.width) {
            blocked[i] = true;
        }
    }
    let sources: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    source::resolve(map, &sources, g.width, g.height, origin, &blocked)
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
    let p = &p.resolved;
    if claimed.len() != cells.len()
        || p.ground >= cells.len()
        || p.indices(g.width).any(|i| claimed[i])
    {
        return false;
    }
    let sources: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    if !p.coherent(&sources, g.width, g.height) {
        return false;
    }
    let ground_uv = g.uv(p.ground % g.width, p.ground / g.width);
    for i in p.indices(g.width) {
        append_top(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            0.,
            ground_uv,
        );
    }
    let rise = p.kind.rise_pixels() * g.tile_height / SOURCE_TILE_HEIGHT;
    if p.kind == Kind::Maze {
        for y in 0..p.height {
            for x in 0..p.width {
                if !p.owns(x, y) {
                    continue;
                }
                let cutaway_start = mesh.solid.positions.len();
                let bounds = g.bounds(p.column + x, p.row + y);
                let triangle_start = mesh.solid.indices.len() / 3;
                let exposed = p.open_mask(x, y);
                crate::dungeon_models::gym_model(4 + usize::from(exposed)).append(
                    &mut mesh.solid,
                    bounds.into(),
                    0.,
                    rise,
                );
                let joins = crate::dungeon_models::gym_wall_join_triangles(
                    exposed, p.covered_join_mask(x, y),
                );
                if !joins.is_empty() {
                    mesh.reveal_join_batches.push(joins.iter()
                        .map(|&ordinal| triangle_start + ordinal).collect());
                }
                // Each owned wall cell is a coherent section. A single large
                // maze range would fade the entire maze for one blocked player.
                if mesh.solid.positions.len() > cutaway_start {
                    mesh.solid.cutaway_ranges.push(cutaway_start..mesh.solid.positions.len());
                }
            }
        }
    } else {
        let (w, _, n, _) = g.bounds(p.column, p.row);
        let e = w + p.width as f32 * g.tile_width;
        let s = n + p.height as f32 * g.tile_height;
        crate::dungeon_models::gym_model(p.kind.asset_index()).append(
            &mut mesh.solid,
            [w, e, n, s],
            0.,
            rise,
        );
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
    fn fixture() -> (Vec<VisualTile>, GridGeometry) {
        let tiles = (0..16)
            .map(|i| VisualTile {
                column: (i % 4) as u32,
                row: (i / 4) as u32,
                source: crystal_render_api::VisualTileSource {
                    tileset_id: Arc::from("elite_four_room"),
                    metatile_id: if i % 4 < 2 && i / 4 < 2 { 0x0f } else { 0x02 },
                    subtile_column: (i % 4) as u8,
                    subtile_row: (i / 4) as u8,
                    tile_index: if i % 4 < 2 && i / 4 < 2 {
                        [[0x07, 0x08], [0x17, 0x18]][i / 4][i % 4]
                    } else {
                        0x03
                    },
                },
                texture: Default::default(),
                priority: false,
            })
            .collect();
        let geometry = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        (tiles, geometry)
    }
    #[test]
    fn complete_gym_planter_reserves_once_and_retains_native_ground() {
        let (tiles, g) = fixture();
        let cells = tiles.iter().collect::<Vec<_>>();
        let mut claimed = vec![false; 16];
        let placements = resolve("GoldenrodGym", &cells, &g, [0, 0], None, &claimed);
        assert_eq!(placements.len(), 1);
        let mut mesh = TerrainMeshData::default();
        mesh.authored_cells = vec![None; 16];
        assert!(append(&mut mesh, &cells, &g, &placements[0], &mut claimed));
        assert_eq!(claimed.iter().filter(|&&v| v).count(), 4);
        assert_eq!(
            mesh.authored_cells.iter().filter(|v| v.is_some()).count(),
            4
        );
        assert!(mesh.textured.positions.iter().all(|p| p[1] == 0.));
        assert!(mesh.textured.cutaway_ranges.is_empty() && mesh.solid.cutaway_ranges.is_empty());
        assert!(mesh.solid.positions.iter().all(|p| p[0] >= 0.
            && p[0] <= 16.001
            && p[1] >= 0.
            && p[1] <= 14.001
            && p[2] >= 0.
            && p[2] <= 16.001));
        let length = mesh.solid.positions.len();
        assert!(!append(&mut mesh, &cells, &g, &placements[0], &mut claimed));
        assert_eq!(mesh.solid.positions.len(), length);
    }
    #[test]
    fn custom_gym_profile_is_authoritative_and_changed_drawing_falls_back() {
        let (mut tiles, g) = fixture();
        let profiles:Document=serde_json::from_str(r#"{"objects":[{"name":"My custom plant","map":"GoldenrodGym","tileset":"elite_four_room","metatile":15,"origin":[0,0],"tiles":[[7,8],[23,24]],"ground":3,"top_pixels":8,"depth_pixels":9}]}"#).unwrap();
        assert!(
            resolve(
                "GoldenrodGym",
                &tiles.iter().collect::<Vec<_>>(),
                &g,
                [0, 0],
                Some(&profiles),
                &[false; 16]
            )
            .is_empty()
        );
        tiles[5].source.tile_index = 0x99;
        assert!(
            resolve(
                "GoldenrodGym",
                &tiles.iter().collect::<Vec<_>>(),
                &g,
                [0, 0],
                None,
                &[false; 16]
            )
            .is_empty()
        );
    }
    #[test]
    fn stale_gym_source_never_appends_partial_geometry() {
        let (mut tiles, g) = fixture();
        let placements = resolve(
            "GoldenrodGym",
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            [0, 0],
            None,
            &[false; 16],
        );
        assert_eq!(placements.len(), 1);
        tiles[5].source.tile_index = 0x99;
        let mut mesh = TerrainMeshData::default();
        let mut claimed = vec![false; 16];
        assert!(!append(
            &mut mesh,
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            &placements[0],
            &mut claimed
        ));
        assert!(mesh.solid.positions.is_empty() && mesh.textured.positions.is_empty());
        assert!(claimed.iter().all(|v| !*v));
    }
}
