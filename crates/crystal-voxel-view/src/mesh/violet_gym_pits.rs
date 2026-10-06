//! Original folded masonry below Violet Gym's exact native blocked recesses.
//! No level-zero underlay, actor footing, collision, warp or world state is
//! created. Every rim is inset into the proven pit side of the source edge.
use super::*;
#[path = "violet_gym_pit_source.rs"]
mod source;
use crate::live_profiles::Document;

const LABEL: &str = "gym:violet-recess-depth";
const DEPTH_PIXELS: f32 = 24.;
pub(super) struct Placement {
    plan: source::Plan,
    blocked: Vec<bool>,
}
impl Placement {
    pub(super) fn indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.plan.cells.iter().copied()
    }
}
fn identities<'a>(cells: &'a [&VisualTile]) -> Vec<source::Identity<'a>> {
    cells
        .iter()
        .map(|tile| {
            let s = &tile.source;
            source::Identity {
                tileset: &s.tileset_id,
                metatile: s.metatile_id,
                column: s.subtile_column,
                row: s.subtile_row,
                tile: s.tile_index,
            }
        })
        .collect()
}
pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Option<Placement> {
    if map != "VioletGym"
        || g.width == 0
        || g.width.checked_mul(g.height) != Some(cells.len())
        || reserved.len() != cells.len()
    {
        return None;
    }
    let mut blocked = reserved.to_vec();
    for p in live::resolve(cells, g.width, g.height, map, profiles) {
        for i in p.indices(g.width) {
            blocked[i] = true;
        }
    }
    let plan = source::resolve(map, &identities(cells), g.width, g.height, &blocked);
    (!plan.cells.is_empty()).then_some(Placement { plan, blocked })
}
fn color(rgb: [f32; 3], factor: f32) -> [f32; 4] {
    [rgb[0] * factor, rgb[1] * factor, rgb[2] * factor, 1.]
}
/// Shared by the production adapter and the lightweight geometry/source audit.
fn append_geometry(mesh: &mut SurfaceMeshData, g: &GridGeometry, plan: &source::Plan) {
    let unit = g.tile_height / SOURCE_TILE_HEIGHT;
    let depth = DEPTH_PIXELS * unit;
    let rim_x = g.tile_width * 0.095;
    let rim_z = g.tile_height * 0.095;
    let mut masks = vec![0u8; g.width * g.height];
    for edge in &plan.edges {
        if edge.rim {
            masks[edge.cell] |= 1 << edge.side;
        }
    }
    for &i in &plan.cells {
        let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
        append_solid_quad(
            mesh,
            [
                [w, -depth, n],
                [w, -depth, s],
                [e, -depth, s],
                [e, -depth, n],
            ],
            [0., 1., 0.],
            [0.018, 0.024, 0.041, 1.],
        );
    }
    for edge in &plan.edges {
        let (w, e, n, s) = g.bounds(edge.cell % g.width, edge.cell / g.width);
        let mask = masks[edge.cell];
        let iw = w + if mask & 8 != 0 { rim_x } else { 0. };
        let ie = e - if mask & 2 != 0 { rim_x } else { 0. };
        let inn = n + if mask & 1 != 0 { rim_z } else { 0. };
        let ins = s - if mask & 4 != 0 { rim_z } else { 0. };
        // Clockwise around the cavity: cross(B-A, down) points inward.
        let (a, b, ia, ib, normal) = match edge.side {
            0 => ([e, n], [w, n], [ie, inn], [iw, inn], [0., 0., 1.]),
            1 => ([e, s], [e, n], [ie, ins], [ie, inn], [-1., 0., 0.]),
            2 => ([w, s], [e, s], [iw, ins], [ie, ins], [0., 0., -1.]),
            _ => ([w, n], [w, s], [iw, inn], [iw, ins], [1., 0., 0.]),
        };
        if edge.rim {
            // Mitered coping stays fully within the original blocked cell;
            // the neighboring native floor and its height remain byte-for-byte.
            append_solid_quad(
                mesh,
                [
                    [a[0], 0., a[1]],
                    [b[0], 0., b[1]],
                    [ib[0], 0., ib[1]],
                    [ia[0], 0., ia[1]],
                ],
                [0., 1., 0.],
                [0.48, 0.53, 0.63, 1.],
            );
        }
        let (a, b) = if edge.rim { (ia, ib) } else { (a, b) };
        let base = if edge.rim {
            [0.27, 0.32, 0.46]
        } else {
            [0.07, 0.09, 0.14]
        };
        // Three broad folded courses and a narrow dark reveal supply readable
        // thickness/depth without source-pixel checkerboard noise or columns.
        for course in 0..3 {
            let top = -(course as f32) * depth / 3.;
            let bottom = -((course + 1) as f32) * depth / 3.;
            let joint = if course == 0 {
                0.95 * unit
            } else {
                0.18 * unit
            };
            let shade = 1. - course as f32 * 0.22;
            append_solid_quad(
                mesh,
                [
                    [a[0], top, a[1]],
                    [b[0], top, b[1]],
                    [b[0], top - joint, b[1]],
                    [a[0], top - joint, a[1]],
                ],
                normal,
                color(
                    if course == 0 {
                        [0.33, 0.39, 0.53]
                    } else {
                        base
                    },
                    shade * 0.68,
                ),
            );
            append_quad_colors(
                mesh,
                [
                    [a[0], top - joint, a[1]],
                    [b[0], top - joint, b[1]],
                    [b[0], bottom, b[1]],
                    [a[0], bottom, a[1]],
                ],
                normal,
                [[0.; 2]; 4],
                [
                    color(base, shade),
                    color(base, shade),
                    color(base, shade * 0.60),
                    color(base, shade * 0.60),
                ],
            );
        }
    }
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: &Placement,
    claimed: &mut [bool],
) -> bool {
    if claimed.len() != cells.len()
        || p.indices().any(|i| claimed[i])
        || source::resolve(
            "VioletGym",
            &identities(cells),
            g.width,
            g.height,
            &p.blocked,
        ) != p.plan
    {
        return false;
    }
    append_geometry(&mut mesh.solid, g, &p.plan);
    for i in p.indices() {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some(LABEL);
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn fixture() -> (Vec<VisualTile>, GridGeometry) {
        let g = GridGeometry {
            width: 8,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let tiles = (0..32)
            .map(|i| {
                let (x, y) = (i % 8, i / 8);
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    texture: Default::default(),
                    priority: false,
                    animation_frames: None,
                    source: VisualTileSource {
                        tileset_id: Arc::from("elite_four_room"),
                        metatile_id: if x < 4 { 0x19 } else { 0x2d },
                        subtile_column: (x % 4) as u8,
                        subtile_row: y as u8,
                        tile_index: if x < 4 {
                            [0x01, 0x01, 0x13, 0x12][y]
                        } else {
                            0x01
                        },
                    },
                }
            })
            .collect();
        (tiles, g)
    }
    #[test]
    fn violet_recess_claims_only_blocked_lower_half_and_never_covers_it_with_floor() {
        let (tiles, g) = fixture();
        let cells = tiles.iter().collect::<Vec<_>>();
        let p = resolve("VioletGym", &cells, &g, None, &[false; 32]).unwrap();
        assert_eq!(
            p.indices().collect::<Vec<_>>(),
            vec![16, 17, 18, 19, 24, 25, 26, 27]
        );
        let mut mesh = TerrainMeshData::default();
        mesh.authored_cells = vec![None; 32];
        mesh.footing_heights = vec![0.; 32];
        let mut claimed = vec![false; 32];
        assert!(append(&mut mesh, &cells, &g, &p, &mut claimed));
        assert_eq!(claimed.iter().filter(|&&v| v).count(), 8);
        assert!(
            mesh.textured.positions.is_empty(),
            "pit may never acquire a floor underlay"
        );
        assert!(mesh.solid.positions.iter().all(|p| p[1] <= 0.));
        assert!(mesh.solid.positions.iter().any(|p| p[1] == -24.));
        assert_eq!(mesh.footing_heights, vec![0.; 32]);
        assert!(
            mesh.solid.cutaway_ranges.is_empty(),
            "existing whole-wall fades remain separate"
        );
        let count = mesh.solid.positions.len();
        assert!(!append(&mut mesh, &cells, &g, &p, &mut claimed));
        assert_eq!(mesh.solid.positions.len(), count);
    }
    #[test]
    fn violet_recess_revalidates_native_floor_guard_before_append() {
        let (mut tiles, g) = fixture();
        let p = resolve(
            "VioletGym",
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            None,
            &[false; 32],
        )
        .unwrap();
        tiles[0].source.tile_index = 0xff;
        let mut mesh = TerrainMeshData::default();
        let mut claimed = vec![false; 32];
        assert!(!append(
            &mut mesh,
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            &p,
            &mut claimed
        ));
        assert!(mesh.solid.positions.is_empty() && claimed.iter().all(|v| !*v));
    }
    #[test]
    fn violet_recess_yields_whole_drawing_to_custom_floor_profile() {
        let (tiles, g) = fixture();
        let cells = tiles.iter().collect::<Vec<_>>();
        let profiles:Document=serde_json::from_str(r#"{"objects":[{"name":"Custom platform","map":"VioletGym","tileset":"elite_four_room","metatile":25,"origin":[0,0],"tiles":[[1,1],[1,1]],"ground":1,"top_pixels":8,"depth_pixels":9}]}"#).unwrap();
        assert!(resolve("VioletGym", &cells, &g, Some(&profiles), &[false; 32]).is_none());
        assert!(resolve("MahoganyGym", &cells, &g, None, &[false; 32]).is_none());
    }
}
