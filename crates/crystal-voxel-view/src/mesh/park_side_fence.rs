//! Complete native side borders; ground, corner hoops and custom art keep priority.
use super::*;
use crate::live_profiles::Document;
#[path = "park_side_fence_source.rs"]
pub(super) mod source;

pub(super) struct Placement {
    column: usize,
    row: usize,
    block: u16,
    ground: [usize; 2],
}
impl Placement {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        let east = source::drawing(self.block).unwrap().1;
        let x = self.column + if east { 2 } else { 0 };
        (self.row..self.row + 4).flat_map(move |y| (x..x + 2).map(move |x| y * width + x))
    }
    fn coherent(&self, cells: &[&VisualTile], g: &GridGeometry) -> bool {
        self.column + 4 <= g.width
            && self.row + 4 <= g.height
            && (0..4).all(|y| {
                (0..4).all(|x| {
                    let s = &cells[(self.row + y) * g.width + self.column + x].source;
                    source::matches_cell(
                        self.block,
                        x,
                        y,
                        (
                            &s.tileset_id,
                            s.metatile_id,
                            s.subtile_column,
                            s.subtile_row,
                            s.tile_index,
                        ),
                    )
                })
            })
    }
}

pub(super) fn ground_sample(cells: &[&VisualTile], paving: bool) -> Option<usize> {
    cells.iter().position(|c| {
        let s = &c.source;
        if paving {
            s.tile_index == 0 && crate::park::masonry_ground(s)
        } else {
            s.tileset_id.as_ref() == "park"
                && s.metatile_id == crate::park::PARK_TREE_GROUND_METATILE
                && s.tile_index == crate::park::PARK_TREE_GROUND_TILE
        }
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
    if !matches!(map, "NationalPark" | "NationalParkBugContest")
        || g.width.checked_mul(g.height) != Some(cells.len())
        || reserved.len() != cells.len()
    {
        return vec![];
    }
    let (Some(paving), Some(lawn)) = (ground_sample(cells, true), ground_sample(cells, false))
    else {
        return vec![];
    };
    let mut out = vec![];
    for c in cells {
        let s = &c.source;
        if s.tileset_id.as_ref() != "park"
            || s.subtile_column != 0
            || s.subtile_row != 0
            || source::drawing(s.metatile_id).is_none()
        {
            continue;
        }
        let p = Placement {
            column: c.column as usize,
            row: c.row as usize,
            block: s.metatile_id,
            ground: [paving, lawn],
        };
        if (origin[0] + p.column as i32).rem_euclid(4) != 0
            || (origin[1] + p.row as i32).rem_euclid(4) != 0
            || !p.coherent(cells, g)
            || p.indices(g.width).any(|i| reserved[i])
        {
            continue;
        }
        // A raw custom override anywhere in the complete drawing wins, even
        // when it only changes a lawn/corner cell outside our eight-cell strip.
        if (0..4).any(|y| {
            (0..4).any(|x| {
                park_scenery::profile_owns_source(
                    map,
                    &cells[(p.row + y) * g.width + p.column + x].source,
                    profiles,
                )
            })
        }) {
            continue;
        }
        out.push(p);
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
        || !p.coherent(cells, g)
        || p.indices(g.width).any(|i| claimed[i])
        || ground_sample(cells, true) != Some(p.ground[0])
        || ground_sample(cells, false) != Some(p.ground[1])
    {
        return false;
    }
    let (_, east, turn) = source::drawing(p.block).unwrap();
    let x = p.column + if east { 2 } else { 0 };
    for i in p.indices(g.width) {
        let outer = if east {
            i % g.width == x + 1
        } else {
            i % g.width == x
        };
        let sample = p.ground[usize::from(!outer)];
        append_top(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            0.,
            g.uv(sample % g.width, sample / g.width),
        );
    }
    let (w, _, n, _) = g.bounds(x, p.row);
    modeled_exteriors::park_hoop_fence::append_side(
        &mut mesh.solid,
        [w, w + 2. * g.tile_width, n, n + 4. * g.tile_height],
        g.tile_height,
        east,
        turn,
    );
    for i in p.indices(g.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some("park/side-hoop-fence");
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(block: u16) -> (VisualWorldFrame, GridGeometry) {
        let drawing = source::drawing(block).unwrap().0;
        let sources = (0..24)
            .map(|i| {
                let x = i % 6;
                let y = i / 6;
                let (b, t) = if x < 4 {
                    (block, drawing[y * 4 + x])
                } else if x == 4 {
                    (1, 0)
                } else {
                    (crate::park::PARK_TREE_GROUND_METATILE, 1)
                };
                let mut s = super::super::tests::source_with_tile(
                    b,
                    if x < 4 { x as u8 } else { 0 },
                    y as u8,
                    t,
                );
                s.tileset_id = "park".into();
                s
            })
            .collect();
        (
            super::super::tests::frame(6, 4, sources),
            GridGeometry {
                width: 6,
                height: 4,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: -16.,
                origin_z: -24.,
            },
        )
    }
    #[test]
    fn park_side_fences_preserve_ground_and_only_claim_their_eight_cells() {
        for block in [0x10, 0x12, 0x14, 0x16, 0x18, 0x1a] {
            let (frame, g) = fixture(block);
            let cells: Vec<_> = frame.tiles.iter().collect();
            let mut claimed = vec![false; 24];
            let out = resolve("NationalPark", &cells, &g, [0, 0], None, &claimed);
            assert_eq!(out.len(), 1);
            let indices: Vec<_> = out[0].indices(g.width).collect();
            let mut mesh = TerrainMeshData {
                authored_cells: vec![None; 24],
                footing_heights: vec![0.; 24],
                ..Default::default()
            };
            assert!(append(&mut mesh, &cells, &g, &out[0], &mut claimed));
            assert_eq!(claimed.iter().filter(|&&v| v).count(), 8);
            assert!(indices.iter().all(|&i| claimed[i]));
            assert_eq!(mesh.textured.quad_count(), 8);
            assert_eq!(mesh.footing_heights, vec![0.; 24]);
            assert!(
                mesh.solid
                    .positions
                    .iter()
                    .all(|p| p.iter().all(|v| v.is_finite()) && p[1] >= 0.)
            );
            assert!(
                mesh.solid
                    .normals
                    .iter()
                    .all(|p| p.iter().all(|v| v.is_finite()))
            );
            assert!(!append(&mut mesh, &cells, &g, &out[0], &mut claimed));
            let (_, east, _) = source::drawing(block).unwrap();
            let x = if east { 0. } else { -16. };
            assert!(
                mesh.solid
                    .positions
                    .iter()
                    .all(|p| p[0] >= x && p[0] <= x + 16. && p[2] >= -24. && p[2] <= 8.)
            );
            for y in 0..4 {
                for x in 0..4 {
                    let mut changed = frame.tiles.clone();
                    changed[y * 6 + x].source.tile_index ^= 1;
                    let cells: Vec<_> = changed.iter().collect();
                    assert!(
                        resolve("NationalPark", &cells, &g, [0, 0], None, &[false; 24]).is_empty()
                    );
                    let mut untouched = TerrainMeshData::default();
                    assert!(!append(
                        &mut untouched,
                        &cells,
                        &g,
                        &out[0],
                        &mut [false; 24]
                    ));
                    assert!(
                        untouched.solid.positions.is_empty()
                            && untouched.textured.positions.is_empty()
                    );
                }
            }
            assert!(resolve("NationalPark", &cells, &g, [1, 0], None, &[false; 24]).is_empty());
            let mut blocked = [false; 24];
            blocked[indices[0]] = true;
            assert!(resolve("NationalPark", &cells, &g, [0, 0], None, &blocked).is_empty());
        }
    }
    #[test]
    fn park_side_fences_defer_to_raw_custom_backing_and_require_both_native_grounds() {
        let (mut frame, g) = fixture(0x14);
        let custom:Document=serde_json::from_str(r#"{"objects":[{"name":"My park backing","map":"NationalPark","tileset":"park","metatile":20,"origin":[2,0],"tiles":[[1]],"ground":65535,"top_pixels":8,"depth_pixels":9}]}"#).unwrap();
        assert!(
            resolve(
                "NationalPark",
                &frame.tiles.iter().collect::<Vec<_>>(),
                &g,
                [0, 0],
                Some(&custom),
                &[false; 24]
            )
            .is_empty()
        );
        for c in &mut frame.tiles {
            if c.source.metatile_id == 1 {
                c.source.tile_index = 0x14;
            }
        }
        assert!(
            resolve(
                "NationalPark",
                &frame.tiles.iter().collect::<Vec<_>>(),
                &g,
                [0, 0],
                None,
                &[false; 24]
            )
            .is_empty()
        );
    }
}
