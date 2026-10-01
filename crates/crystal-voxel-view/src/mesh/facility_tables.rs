//! Original source-complete facility tables and square-backed chairs.
use super::*;
use crate::interior_models::Model;
use crate::live_profiles::Document;
use std::sync::OnceLock;
#[path = "facility_tables_source.rs"]
mod source;
use source::{Identity, Kind};
pub(super) struct Placement {
    resolved: source::Match,
}
impl Placement {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        self.resolved.indices(width)
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
fn model(kind: Kind) -> &'static Model {
    static MODELS: OnceLock<[Model; 4]> = OnceLock::new();
    &MODELS.get_or_init(|| {
        [
            include_str!("../../models/facility_tables/facility_square_document_table.mesh.json"),
            include_str!("../../models/facility_tables/facility_meeting_table.mesh.json"),
            include_str!("../../models/facility_tables/facility_document_side_desk.mesh.json"),
            include_str!("../../models/facility_tables/facility_square_back_chair.mesh.json"),
        ]
        .map(|s| Model::parse(s).expect("validated original facility table mesh"))
    })[kind as usize]
}
pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    if g.width.checked_mul(g.height) != Some(cells.len()) || reserved.len() != cells.len() {
        return Vec::new();
    }
    let mut customized = vec![false; cells.len()];
    // All live profiles win, including renamed profiles with identical source.
    // Existing generic signature models are replaced only after this resolves.
    for p in live::resolve(cells, g.width, g.height, map, profiles) {
        for i in p.indices(g.width) {
            customized[i] = true;
        }
    }
    let identities: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    source::resolve(
        map,
        &identities,
        g.width,
        g.height,
        origin,
        reserved,
        &customized,
    )
    .into_iter()
    .map(|resolved| Placement { resolved })
    .collect()
}
/// Only the small original document ink face stays live on a physical page.
/// Pixel clipping splits at source-cell boundaries so UVs do not cross tiles.
fn live_panel(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &source::Match,
    pixels: [f32; 4],
    corners: [[f32; 3]; 4],
    normal: [f32; 3],
) {
    let [sx, sy, sw, sh] = pixels;
    let mix = |u: f32, v: f32| -> [f32; 3] {
        std::array::from_fn(|i| {
            corners[0][i] * (1. - u) * (1. - v)
                + corners[1][i] * u * (1. - v)
                + corners[2][i] * u * v
                + corners[3][i] * (1. - u) * v
        })
    };
    let mut y = sy;
    while y < sy + sh {
        let y1 = (sy + sh).min(((y / 8.).floor() + 1.) * 8.);
        let mut x = sx;
        while x < sx + sw {
            let x1 = (sx + sw).min(((x / 8.).floor() + 1.) * 8.);
            // Compute all UVs from this cell, including its far edge.
            let tx = (x / 8.).floor() as usize;
            let ty = (y / 8.).floor() as usize;
            let uv = g.uv(p.column + tx, p.row + ty);
            let u = |v: f32| uv.0 + (uv.1 - uv.0) * (v - tx as f32 * 8.) / 8.;
            let v = |v: f32| uv.2 + (uv.3 - uv.2) * (v - ty as f32 * 8.) / 8.;
            append_quad(
                &mut mesh.textured,
                [
                    mix((x - sx) / sw, (y1 - sy) / sh),
                    mix((x1 - sx) / sw, (y1 - sy) / sh),
                    mix((x1 - sx) / sw, (y - sy) / sh),
                    mix((x - sx) / sw, (y - sy) / sh),
                ],
                normal,
                [[u(x), v(y1)], [u(x1), v(y1)], [u(x1), v(y)], [u(x), v(y)]],
                TEXTURED_SHADE,
            );
            x = x1;
        }
        y = y1;
    }
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    placement: &Placement,
    claimed: &mut [bool],
) -> bool {
    let p = &placement.resolved;
    if claimed.len() != cells.len() || p.indices(g.width).any(|i| i >= claimed.len() || claimed[i])
    {
        return false;
    }
    let identities: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    if !p.coherent(&identities, g.width, g.height) {
        return false;
    }
    let (west, _, north, _) = g.bounds(p.column, p.row);
    let sx = g.tile_width / 8.;
    let sy = g.tile_height / 8.;
    for i in p.indices(g.width) {
        let sample = p.ground_for(i % g.width, i / g.width);
        append_top(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            0.,
            g.uv(sample % g.width, sample / g.width),
        );
    }
    let ([w, e, n, s], rise) = p.kind().fitting();
    let bounds = [west + w * sx, west + e * sx, north + n * sy, north + s * sy];
    model(p.kind()).append_fitted(&mut mesh.solid, bounds, 0., rise * sy);
    if matches!(p.kind(), Kind::SquareTable | Kind::SideDesk) {
        let (source_y, paper_z) = if p.kind() == Kind::SquareTable {
            (17., 18.36)
        } else {
            (9., 14.36)
        };
        let point = |x: f32, z: f32| [west + x * sx, 9.835 * sy, north + z * sy];
        live_panel(
            mesh,
            g,
            p,
            [9., source_y, 6., 6.],
            [
                point(9.05, paper_z),
                point(15., paper_z),
                point(15., paper_z + 6.16),
                point(9.05, paper_z + 6.16),
            ],
            [0., 1., 0.],
        );
    }
    for i in p.indices(g.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some(p.kind().label());
        }
    }
    true
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    include!("facility_tables_tests.rs");
}
