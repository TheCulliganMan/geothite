//! Complete source-aware store architecture and closed fifth-floor islands.
//! Sparse claims preserve live stairs, lift doors, thresholds, shop surfaces,
//! and the small genuine floor inset above the closed lower display.
use super::*;
use crate::interior_models::Model;
use crate::live_profiles::Document;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Asset {
    Window1,
    Window2,
    Window4,
    Window8,
    Window10,
    LiftSide,
    Directory,
    ClosedU,
}
impl Asset {
    fn model(self) -> &'static Model {
        static MODELS: OnceLock<[Model; 8]> = OnceLock::new();
        &MODELS.get_or_init(|| {
            [
                crate::model_storage::include_model!(
                    "models/department_store/window_course_1.mesh.json"
                ),
                crate::model_storage::include_model!(
                    "models/department_store/window_course_2.mesh.json"
                ),
                crate::model_storage::include_model!(
                    "models/department_store/window_course_4.mesh.json"
                ),
                crate::model_storage::include_model!(
                    "models/department_store/window_course_8.mesh.json"
                ),
                crate::model_storage::include_model!(
                    "models/department_store/window_course_10.mesh.json"
                ),
                crate::model_storage::include_model!(
                    "models/department_store/lift_side.mesh.json"
                ),
                crate::model_storage::include_model!(
                    "models/department_store/directory.mesh.json"
                ),
                crate::model_storage::include_model!(
                    "models/department_store/closed_u_display.mesh.json"
                ),
            ]
            .map(|s| Model::parse(s).expect("validated department store kit"))
        })[self as usize]
    }
}
struct Part {
    x: usize,
    y: usize,
    width: usize,
    depth: usize,
    asset: Asset,
}
struct Network {
    floor: u8,
    anchor: [i32; 2],
    width: usize,
    height: usize,
    fingerprint: u64,
    rows: &'static [u32],
    parts: &'static [Part],
    label: &'static str,
}
include!("department_store_bindings.rs");
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
                    (bits & (1u32 << x) != 0).then_some((self.row + y) * width + self.column + x)
                })
            })
    }
}
fn floor(map: &str) -> Option<u8> {
    let floor = map
        .strip_prefix("CeladonDeptStore")
        .or_else(|| map.strip_prefix("GoldenrodDeptStore"))?;
    match floor {
        "1F" => Some(1),
        "2F" => Some(2),
        "3F" => Some(3),
        "4F" => Some(4),
        "5F" => Some(5),
        "6F" => Some(6),
        _ => None,
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
    let sources = || {
        (0..n.height).flat_map(move |dy| {
            (0..n.width).map(move |dx| &cells[(y + dy) * g.width + x + dx].source)
        })
    };
    sources().all(|s| s.tileset_id.as_ref() == "mart") && identity_hash(sources()) == n.fingerprint
}
fn valid_ground(cell: &VisualTile) -> bool {
    let s = &cell.source;
    s.tileset_id.as_ref() == "mart"
        && s.metatile_id == 0x04
        && s.tile_index == 0x01
        && s.subtile_column < 4
        && s.subtile_row < 4
}
pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    let Some(floor) = floor(map) else {
        return Vec::new();
    };
    resolve_networks(NETWORKS, floor, map, cells, g, origin, profiles, reserved)
}
fn resolve_networks(
    networks: &'static [Network],
    floor: u8,
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
    // No live profile is silently displaced. The bundled Mart furniture lies
    // outside this kit's sparse masks; added or changed profiles win overlaps.
    let mut blocked = reserved.to_vec();
    for p in live::resolve(cells, g.width, g.height, map, profiles) {
        for i in p.indices(g.width) {
            blocked[i] = true;
        }
    }
    let mut out = Vec::new();
    for n in networks.iter().filter(|n| n.floor == floor) {
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
// Tile rectangles remain live atlas samples, folded into the original housing.
fn inscription(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    source: [usize; 2],
    size: [usize; 2],
    bounds: [f32; 4],
    south: f32,
) {
    let [left, right, bottom, top] = bounds;
    for y in 0..size[1] {
        for x in 0..size[0] {
            let w = left + (right - left) * x as f32 / size[0] as f32;
            let e = left + (right - left) * (x + 1) as f32 / size[0] as f32;
            let hi = top - (top - bottom) * y as f32 / size[1] as f32;
            let lo = top - (top - bottom) * (y + 1) as f32 / size[1] as f32;
            let (u0, u1, v0, v1) = g.uv(source[0] + x, source[1] + y);
            append_quad(
                &mut mesh.textured,
                [
                    [w, lo, south],
                    [e, lo, south],
                    [e, hi, south],
                    [w, hi, south],
                ],
                [0., 0., 1.],
                [[u0, v1], [u1, v1], [u1, v0], [u0, v0]],
                TEXTURED_SHADE,
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
        append_top(&mut mesh.textured, [w, e, n, s], 0., uv);
    }
    let cutaway_start = mesh.solid.positions.len();
    let textured_start = mesh.textured.positions.len();
    let sx = g.tile_width / 8.;
    let sy = g.tile_height / 8.;
    for part in p.network.parts {
        let (w, _, n, _) = g.bounds(p.column + part.x, p.row + part.y);
        let e = w + part.width as f32 * g.tile_width;
        let s = n + part.depth as f32 * g.tile_height;
        let is_display = part.asset == Asset::ClosedU;
        let north = if is_display { n } else { s - 3. * sy };
        part.asset.model().append_fitted(
            &mut mesh.solid,
            [w, e, north, s],
            0.,
            if is_display { 8. * sy } else { 16. * sy },
        );
        if part.asset == Asset::Directory {
            inscription(
                mesh,
                g,
                [p.column + part.x, p.row + part.y],
                [2, 2],
                [w + 1.5 * sx, e - 1.5 * sx, 1.5 * sy, 14.5 * sy],
                s + 0.015 * sy,
            );
        } else if part.asset == Asset::LiftSide {
            inscription(
                mesh,
                g,
                [p.column + part.x, p.row + part.y],
                [1, 1],
                [w + 1.5 * sx, w + 7. * sx, 8. * sy, 13.5 * sy],
                s + 0.015 * sy,
            );
        }
    }
    // Reveal is presentation-only and leaves source support/collision untouched.
    mesh.solid
        .cutaway_ranges
        .push(cutaway_start..mesh.solid.positions.len());
    if mesh.textured.positions.len() > textured_start {
        mesh.textured
            .cutaway_ranges
            .push(textured_start..mesh.textured.positions.len());
        mesh.cutaway_links.push((cutaway_start, textured_start));
    }
    for i in p.indices(g.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some(p.network.label);
        }
    }
    true
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    include!("department_store_tests.rs");
}
