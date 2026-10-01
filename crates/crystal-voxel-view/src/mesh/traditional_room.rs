//! Original open slatted partitions and one coherent native-height stage.
use super::*;
use crate::live_profiles::Document;
#[path = "traditional_room_source.rs"]
mod source;
use source::{Identity, Kind};
pub(super) struct Placement {
    resolved: source::Match,
}
impl Placement {
    pub(super) fn indices(&self, w: usize) -> impl Iterator<Item = usize> + '_ {
        self.resolved.indices(w)
    }
}
fn identity(s: &VisualTileSource) -> Identity<'_> {
    Identity {
        tileset: &s.tileset_id,
        metatile: s.metatile_id,
        column: s.subtile_column,
        row: s.subtile_row,
        tile: s.tile_index,
    }
}
pub(super) fn tatami_source(map: &str, s: &VisualTileSource) -> bool {
    source::tatami_cell(map, &identity(s))
}
pub(super) fn floor_map(map: &str) -> bool {
    matches!(map, "DanceTheater" | "KurtsHouse")
}
pub(super) fn tatami_floor_mask(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    excluded: &[bool],
) -> Vec<bool> {
    let mut mask = vec![false; cells.len()];
    if !floor_map(map)
        || g.width.checked_mul(g.height) != Some(cells.len())
        || (!excluded.is_empty() && excluded.len() != cells.len())
    {
        return mask;
    }
    for y in 0..g.height.saturating_sub(3) {
        for x in 0..g.width.saturating_sub(3) {
            if (origin[0] + x as i32).rem_euclid(4) != 0
                || (origin[1] + y as i32).rem_euclid(4) != 0
            {
                continue;
            }
            let full = (0..4).all(|dy| {
                (0..4).all(|dx| {
                    let i = (y + dy) * g.width + x + dx;
                    let s = &cells[i].source;
                    tatami_source(map, s)
                        && usize::from(s.subtile_column) == dx
                        && usize::from(s.subtile_row) == dy
                        && !excluded.get(i).copied().unwrap_or(false)
                        && matches!(
                            shape_for_source_on_map(map, s),
                            CellShape::Flat | CellShape::PlaneAt { height: 0.0 }
                        )
                })
            });
            if full {
                for dy in 0..4 {
                    for dx in 0..4 {
                        mask[(y + dy) * g.width + x + dx] = true;
                    }
                }
            }
        }
    }
    mask
}
pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    if !matches!(map, "WiseTriosRoom" | "Route39Barn" | "DanceTheater")
        || g.width.checked_mul(g.height) != Some(cells.len())
        || reserved.len() != cells.len()
    {
        return vec![];
    }
    let mut blocked = reserved.to_vec();
    // Every resolving custom profile is authoritative, including guard gaps.
    for p in live::resolve(cells, g.width, g.height, map, profiles) {
        for i in p.indices(g.width) {
            blocked[i] = true;
        }
    }
    let sources: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    source::resolve(map, &sources, g.width, g.height, origin, &blocked)
        .into_iter()
        .filter(|p| {
            matches!(
                shape_for_source_on_map(map, &cells[p.ground].source),
                CellShape::Flat | CellShape::PlaneAt { height: 0.0 }
            )
        })
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
    if claimed.len() != cells.len() || g.width.checked_mul(g.height) != Some(cells.len()) {
        return false;
    }
    let sources: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    if !p.coherent(&sources, g.width, g.height) || p.indices(g.width).any(|i| claimed[i]) {
        return false;
    }
    let scale = g.tile_height / SOURCE_TILE_HEIGHT;
    let (west, _, north, _) = g.bounds(p.column, p.row);
    if p.kind == Kind::Theater {
        // Entire top stays precisely 8px high, including source access columns.
        crate::dungeon_models::traditional_model(1).append(
            &mut mesh.solid,
            [
                west,
                west + 24. * g.tile_width,
                north,
                north + 10. * g.tile_height,
            ],
            0.,
            8. * scale,
        );
        crate::dungeon_models::traditional_model(2).append(
            &mut mesh.solid,
            [
                west,
                west + 24. * g.tile_width,
                north + 1.5 * g.tile_height,
                north + 2. * g.tile_height,
            ],
            8. * scale,
            16. * scale,
        );
    } else {
        let uv = g.uv(p.ground % g.width, p.ground / g.width);
        for i in p.indices(g.width) {
            append_top(
                &mut mesh.textured,
                g.bounds(i % g.width, i / g.width).into(),
                0.,
                uv,
            );
        }
        let mut rail = |x: usize, y: usize, width: usize| {
            let z = north + (y + 2) as f32 * g.tile_height;
            for dx in 0..width {
                let x0 = west + (x + dx) as f32 * g.tile_width;
                crate::dungeon_models::traditional_model(0).append(
                    &mut mesh.solid,
                    [x0, x0 + g.tile_width, z - 2.5 * scale, z],
                    0.,
                    16. * scale,
                );
            }
        };
        match p.kind {
            Kind::WiseCourse => rail(0, 0, 4),
            Kind::WiseJoint => {
                rail(0, 0, 4);
                rail(2, 2, 2);
            }
            Kind::WiseReturn => rail(2, 2, 2),
            Kind::BarnRail => rail(0, 2, 4),
            Kind::Theater => unreachable!(),
        }
    }
    // Model::append applies the shared authored face lighting exactly once.
    for y in 0..p.height {
        for x in 0..p.width {
            if p.owns(x, y) {
                let i = (p.row + y) * g.width + p.column + x;
                claimed[i] = true;
                if mesh.authored_cells.len() == cells.len() {
                    mesh.authored_cells[i] = p.object_label(x, y);
                }
            }
        }
    }
    // Deliberately never edit footing heights, collision, actor state or cuts.
    true
}
#[cfg(test)]
#[path = "traditional_room_tests.rs"]
mod tests;
