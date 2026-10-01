//! Exact Cable Club source networks. Sparse ownership preserves all booth,
//! reception, source inscription, Time Capsule doorway and actor openings.
use super::*;
use crate::live_profiles::Document;

struct Network {
    anchor: [i32; 2],
    width: usize,
    height: usize,
    fingerprint: u64,
    rows: &'static [u16],
    // x, y, width, depth in source cells; contiguous long and short shells.
    shells: &'static [[usize; 4]],
    record_sign: bool,
}
include!("cable_club_bindings.rs");

pub(super) struct Placement {
    network: &'static Network,
    column: usize,
    row: usize,
    ground: usize,
    grid_origin: [i32; 2],
}
impl Placement {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        self.network
            .rows
            .iter()
            .enumerate()
            .flat_map(move |(y, bits)| {
                (0..self.network.width).filter_map(move |x| {
                    (bits & (1u16 << x) != 0).then_some((self.row + y) * width + self.column + x)
                })
            })
    }
}
fn identity_hash<'a>(sources: impl IntoIterator<Item = &'a VisualTileSource>) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for s in sources {
        for byte in [
            (s.metatile_id & 255) as u8,
            (s.metatile_id >> 8) as u8,
            s.subtile_column,
            s.subtile_row,
            (s.tile_index & 255) as u8,
            (s.tile_index >> 8) as u8,
        ] {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
    }
    hash
}
fn complete(
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    x: usize,
    y: usize,
    n: &Network,
) -> bool {
    if cells.len() != g.width * g.height
        || x + n.width > g.width
        || y + n.height > g.height
        || i64::from(origin[0]) + x as i64 != i64::from(n.anchor[0])
        || i64::from(origin[1]) + y as i64 != i64::from(n.anchor[1])
    {
        return false;
    }
    // The guard rectangle includes both native phase and the unowned opening.
    // Source-map north/east edges are explicit; viewport edges are never used.
    let sources = || {
        (0..n.height).flat_map(move |dy| {
            (0..n.width).map(move |dx| &cells[(y + dy) * g.width + x + dx].source)
        })
    };
    sources().all(|s| s.tileset_id.as_ref() == "pokecenter")
        && identity_hash(sources()) == n.fingerprint
}
fn valid_ground(cell: &VisualTile) -> bool {
    let s = &cell.source;
    s.tileset_id.as_ref() == "pokecenter"
        && s.metatile_id == 0x04
        && s.tile_index == 0x11
        && s.subtile_column < 4
        && s.subtile_row < 4
}
fn canonical_partition(o: &crate::live_profiles::Object) -> bool {
    if !matches!(
        o.name.as_str(),
        "Pokecenter2F booth partition 31-32"
            | "Pokecenter2F booth partition 0b-28"
            | "Pokecenter2F booth partition 0b-0f"
    ) {
        return false;
    }
    static DOC: std::sync::OnceLock<Document> = std::sync::OnceLock::new();
    DOC.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../../modpacks/voxel-view/profiles.json"
        ))
        .expect("valid bundled profiles")
    })
    .objects
    .iter()
    .any(|canonical| canonical == o)
}
pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    if !MAPS.contains(&map) {
        return Vec::new();
    }
    resolve_networks(NETWORKS, map, cells, g, origin, profiles, reserved)
}
fn resolve_networks(
    networks: &'static [Network],
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    if cells.len() != g.width * g.height || reserved.len() != cells.len() {
        return Vec::new();
    }
    let Some(ground) = cells.iter().position(|c| valid_ground(c)) else {
        return Vec::new();
    };
    let custom = profiles.map(|doc| Document {
        objects: doc
            .objects
            .iter()
            .filter(|o| !canonical_partition(o))
            .cloned()
            .collect(),
        atmosphere: None,
    });
    let mut blocked = reserved.to_vec();
    for p in live::resolve(cells, g.width, g.height, map, custom.as_ref()) {
        for i in p.indices(g.width) {
            blocked[i] = true;
        }
    }
    let mut out = Vec::new();
    for n in networks {
        let x = i64::from(n.anchor[0]) - i64::from(origin[0]);
        let y = i64::from(n.anchor[1]) - i64::from(origin[1]);
        if x < 0 || y < 0 {
            continue;
        }
        let (column, row) = (x as usize, y as usize);
        if !complete(cells, g, origin, column, row, n) {
            continue;
        }
        let p = Placement {
            network: n,
            column,
            row,
            ground,
            grid_origin: origin,
        };
        if p.indices(g.width).any(|i| blocked[i]) {
            continue;
        }
        for i in p.indices(g.width) {
            blocked[i] = true;
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
        || p.ground >= cells.len()
        || !valid_ground(cells[p.ground])
        || !complete(cells, g, p.grid_origin, p.column, p.row, p.network)
        || p.indices(g.width).any(|i| claimed[i])
    {
        return false;
    }
    let uv = g.uv(
        cells[p.ground].column as usize,
        cells[p.ground].row as usize,
    );
    for i in p.indices(g.width) {
        let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
        append_top(&mut mesh.textured, [w, e, n, s], 0.0, uv);
    }
    for &[dx, dy, width, depth] in p.network.shells {
        let (west, _, north, _) = g.bounds(p.column + dx, p.row + dy);
        let model = if depth == 8 {
            crate::cable_club_models::divider()
        } else {
            crate::cable_club_models::short_return()
        };
        model.append_fitted(
            &mut mesh.solid,
            [
                west,
                west + width as f32 * g.tile_width,
                north,
                north + depth as f32 * g.tile_height,
            ],
            0.0,
            16.0 * g.tile_height / 8.0,
        );
    }
    if p.network.record_sign {
        // The live record inscription remains readable on the divider's south
        // end, within its footprint. It is never baked into the original kit.
        for dx in 0..2 {
            let x = p.column + 1 + dx;
            let (west, east, _, _) = g.bounds(x, p.row);
            let south = g.bounds(x, p.row + 7).3 - 0.005 * g.tile_height;
            let (u0, u1, v0, v1) = g.uv(x, p.row + 6);
            append_quad(
                &mut mesh.textured,
                [
                    [west, 5.0 * g.tile_height / 8.0, south],
                    [east, 5.0 * g.tile_height / 8.0, south],
                    [east, 13.0 * g.tile_height / 8.0, south],
                    [west, 13.0 * g.tile_height / 8.0, south],
                ],
                [0.0, 0.0, 1.0],
                [[u0, v1], [u1, v1], [u1, v0], [u0, v0]],
                TEXTURED_SHADE,
            );
        }
    }
    for i in p.indices(g.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some("pokecenter/cable_club_partition");
        }
    }
    true
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    include!("cable_club_tests.rs");
}
