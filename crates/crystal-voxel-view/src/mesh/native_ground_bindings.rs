//! Exact source/backing exceptions for three native maps without lawn art.
//! Reuses the existing stone/grass meshes; never edits collision or footing.
use super::*;
use crate::live_profiles::Document;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    LandRock,
    Grass,
}
#[derive(Clone, Copy, Debug)]
struct Binding {
    map: &'static str,
    map_size: [i32; 2],
    tileset: &'static str,
    block: u16,
    origin: [u8; 2],
    size: usize,
    kind: Kind,
    preferred_ground: u16,
    ground_block: u16,
    ground_tile: u16,
}
impl Binding {
    fn water_bed(&self) -> bool {
        self.map == "Route19" && self.ground_block == 0x43 && self.ground_tile == 0x14
    }
    fn backing_height(&self, tile_height: f32) -> f32 {
        if self.water_bed() {
            CellShape::Water.surface_height(tile_height)
        } else {
            0.0
        }
    }
}
const BINDINGS: &[Binding] = &[
    Binding {
        map: "Route10South",
        map_size: [40, 36],
        tileset: "kanto",
        block: 0x61,
        origin: [0, 2],
        size: 2,
        kind: Kind::LandRock,
        preferred_ground: 0x2c,
        ground_block: 0x31,
        ground_tile: 0x39,
    },
    Binding {
        map: "Route10South",
        map_size: [40, 36],
        tileset: "kanto",
        block: 0x61,
        origin: [2, 2],
        size: 2,
        kind: Kind::LandRock,
        preferred_ground: 0x2c,
        ground_block: 0x31,
        ground_tile: 0x39,
    },
    Binding {
        map: "Route19",
        map_size: [40, 72],
        tileset: "kanto",
        block: 0x13,
        origin: [0, 0],
        size: 2,
        kind: Kind::LandRock,
        preferred_ground: 0x2c,
        ground_block: 0x43,
        ground_tile: 0x14,
    },
    Binding {
        map: "Route19",
        map_size: [40, 72],
        tileset: "kanto",
        block: 0x13,
        origin: [2, 0],
        size: 2,
        kind: Kind::LandRock,
        preferred_ground: 0x2c,
        ground_block: 0x43,
        ground_tile: 0x14,
    },
    Binding {
        map: "Route19",
        map_size: [40, 72],
        tileset: "kanto",
        block: 0x13,
        origin: [0, 2],
        size: 2,
        kind: Kind::LandRock,
        preferred_ground: 0x2c,
        ground_block: 0x43,
        ground_tile: 0x14,
    },
    Binding {
        map: "Route19",
        map_size: [40, 72],
        tileset: "kanto",
        block: 0x13,
        origin: [2, 2],
        size: 2,
        kind: Kind::LandRock,
        preferred_ground: 0x2c,
        ground_block: 0x43,
        ground_tile: 0x14,
    },
    Binding {
        map: "RuinsOfAlphOutside",
        map_size: [40, 72],
        tileset: "johto",
        block: 0x03,
        origin: [0, 0],
        size: 4,
        kind: Kind::Grass,
        preferred_ground: 0x05,
        ground_block: 0x01,
        ground_tile: 0x06,
    },
];
pub(super) struct Placement {
    binding: &'static Binding,
    column: usize,
    row: usize,
    ground: usize,
    grid_origin: [i32; 2],
}
impl Placement {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        (0..self.binding.size).flat_map(move |dy| {
            (0..self.binding.size).map(move |dx| (self.row + dy) * width + self.column + dx)
        })
    }
}
fn native_cell(cell: &VisualTile, b: &Binding, grid_origin: [i32; 2]) -> bool {
    let x = i64::from(grid_origin[0]) + i64::from(cell.column);
    let y = i64::from(grid_origin[1]) + i64::from(cell.row);
    x >= 0
        && y >= 0
        && x < i64::from(b.map_size[0])
        && y < i64::from(b.map_size[1])
        && cell.source.tileset_id.as_ref() == b.tileset
        && i64::from(cell.source.subtile_column) == x % 4
        && i64::from(cell.source.subtile_row) == y % 4
}
fn native_ground(cell: &VisualTile, b: &Binding, origin: [i32; 2]) -> bool {
    native_cell(cell, b, origin)
        && cell.source.metatile_id == b.ground_block
        && cell.source.tile_index == b.ground_tile
        && if b.water_bed() {
            shape_for_source_on_map(b.map, &cell.source) == CellShape::Water
        } else {
            matches!(
                shape_for_source_on_map(b.map, &cell.source),
                CellShape::Flat | CellShape::PlaneAt { height: 0.0 }
            )
        }
}
fn complete(cells: &[&VisualTile], g: &GridGeometry, p: &Placement) -> bool {
    let b = p.binding;
    cells.len() == g.width * g.height
        && p.column + b.size <= g.width
        && p.row + b.size <= g.height
        && p.indices(g.width).enumerate().all(|(local, i)| {
            let cell = cells[i];
            let dx = local % b.size;
            let dy = local / b.size;
            let expected = match b.kind {
                Kind::LandRock => [[0x2a, 0x2b], [0x3a, 0x3b]][dy][dx],
                Kind::Grass => 0x04,
            };
            native_cell(cell, b, p.grid_origin)
                && cell.source.metatile_id == b.block
                && cell.source.subtile_column == b.origin[0] + dx as u8
                && cell.source.subtile_row == b.origin[1] + dy as u8
                && cell.source.tile_index == expected
        })
}
// Protect even a cropped custom object whose own ground is currently absent.
// A same-art custom profile may own a different height, palette or silhouette.
fn profile_owns(map: &str, source: &VisualTileSource, profiles: Option<&Document>) -> bool {
    profiles.is_some_and(|d| {
        d.objects.iter().any(|o| {
            o.map.as_deref().is_none_or(|m| m == map)
                && o.maps
                    .as_ref()
                    .is_none_or(|maps| maps.iter().any(|m| m == map))
                && o.tileset == source.tileset_id.as_ref()
                && o.tiles.iter().enumerate().any(|(y, row)| {
                    row.iter().enumerate().any(|(x, &tile)| {
                        let sx = usize::from(o.origin[0]) + x;
                        let sy = usize::from(o.origin[1]) + y;
                        let block = o
                            .metatiles
                            .as_ref()
                            .and_then(|rows| rows.get(sy / 4))
                            .and_then(|row| row.get(sx / 4))
                            .copied()
                            .unwrap_or(o.metatile);
                        source.metatile_id == block
                            && usize::from(source.subtile_column) == sx % 4
                            && usize::from(source.subtile_row) == sy % 4
                            && source.tile_index == tile
                    })
                })
        })
    })
}
pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    grid_origin: [i32; 2],
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    if cells.len() != g.width * g.height || cells.len() != reserved.len() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut blocked = reserved.to_vec();
    for b in BINDINGS.iter().filter(|b| b.map == map) {
        // A normal same-map lawn sample keeps the original adapter in charge.
        if cells.iter().any(|c| {
            native_cell(c, b, grid_origin)
                && c.source.tile_index == b.preferred_ground
                && matches!(
                    shape_for_source_on_map(map, &c.source),
                    CellShape::Flat | CellShape::PlaneAt { height: 0.0 }
                )
        }) {
            continue;
        }
        let Some(ground) = cells.iter().position(|c| {
            native_ground(c, b, grid_origin) && !profile_owns(map, &c.source, profiles)
        }) else {
            continue;
        };
        for row in 0..g.height {
            for column in 0..g.width {
                let p = Placement {
                    binding: b,
                    column,
                    row,
                    ground,
                    grid_origin,
                };
                if !complete(cells, g, &p)
                    || p.indices(g.width)
                        .any(|i| blocked[i] || profile_owns(map, &cells[i].source, profiles))
                {
                    continue;
                }
                for i in p.indices(g.width) {
                    blocked[i] = true;
                }
                result.push(p);
            }
        }
    }
    result
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: &Placement,
    claimed: &mut [bool],
) -> bool {
    if claimed.len() != cells.len()
        || !complete(cells, g, p)
        || p.indices(g.width).any(|i| claimed[i])
        || p.ground >= cells.len()
        || !native_ground(cells[p.ground], p.binding, p.grid_origin)
    {
        return false;
    }
    let uv = g.uv(p.ground % g.width, p.ground / g.width);
    for i in p.indices(g.width) {
        let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
        append_top(
            &mut mesh.textured,
            [w, e, n, s],
            p.binding.backing_height(g.tile_height),
            uv,
        );
    }
    let label = match p.binding.kind {
        Kind::LandRock => {
            // Identical fit, rise, cached mesh and palette as kanto_boundary_rocks.
            let kind = crate::exterior_models::Kind::KantoBoundaryLand;
            let m = crate::exterior_models::model(kind);
            let (w, _, n, _) = g.bounds(p.column, p.row);
            m.append_fitted(
                &mut mesh.solid,
                [
                    w + g.tile_width * 0.06,
                    (w + g.tile_width * 2.0) - g.tile_width * 0.06,
                    n + g.tile_height * 0.09,
                    (n + g.tile_height * 2.0) - g.tile_height * 0.09,
                ],
                0.0,
                g.tile_height * 1.55 / (m.max[1] - m.min[1]),
                None,
            );
            kind.label()
        }
        Kind::Grass => {
            for i in p.indices(g.width) {
                let appended = modeled_exteriors::append_grass(
                    &mut mesh.solid,
                    p.binding.map,
                    cells[i],
                    g,
                    0.0,
                    p.grid_origin,
                );
                debug_assert!(appended, "verified Johto grass source must append");
            }
            "johto/tall_grass"
        }
    };
    for i in p.indices(g.width) {
        claimed[i] = true;
        mesh.authored_cells[i] = Some(label);
    }
    true
}
#[cfg(test)]
#[path = "native_ground_bindings_tests.rs"]
mod tests;
