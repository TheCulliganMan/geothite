//! Original source-complete institutional benches and connected radio desks.
use super::*;
use crate::interior_models::{Model, ModelKind};
use crate::live_profiles::Document;
use std::sync::OnceLock;
#[path = "facility_radio_source.rs"]
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
fn model(index: usize) -> &'static Model {
    static MODELS: OnceLock<[Model; 6]> = OnceLock::new();
    &MODELS.get_or_init(|| {
        [
            crate::model_storage::include_model!(
                "models/facility_radio/facility_workbench.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/facility_radio/radio_phone_desk.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/facility_radio/radio_memo_desk.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/facility_radio/radio_reception_u.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/facility_radio/radio_reception_l.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/facility_radio/radio_counter_extension.mesh.json"
            ),
        ]
        .map(|s| Model::parse(s).expect("validated original facility and radio mesh"))
    })[index]
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
fn pixel_uv(g: &GridGeometry, p: &source::Match, x: f32, y: f32) -> [f32; 2] {
    let tx = (x / 8.).floor() as usize;
    let ty = (y / 8.).floor() as usize;
    let uv = g.uv(p.column + tx, p.row + ty);
    [
        uv.0 + (uv.1 - uv.0) * (x - tx as f32 * 8.) / 8.,
        uv.2 + (uv.3 - uv.2) * (y - ty as f32 * 8.) / 8.,
    ]
}
/// Only the source readout/memo face stays live, never a copied cabinet skin.
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
    match p.kind() {
        Kind::InstrumentBank => crate::dungeon_models::room_model(
            crate::dungeon_models::RoomAsset::FacilityInstrumentBank,
        )
        .append_source_sampled(&mut mesh.textured, bounds, 0., rise * sy, |v| {
            pixel_uv(g, p, v[0] + 0.5, v[1] + 0.5)
        }),
        Kind::Stool | Kind::RoundStool => crate::interior_models::model(ModelKind::ArcadeStool)
            .append_fitted(&mut mesh.solid, bounds, 0., rise * sy),
        kind => model(kind.model_index()).append_fitted(&mut mesh.solid, bounds, 0., rise * sy),
    }
    let point = |x: f32, y: f32, z: f32| [west + x * sx, y * sy, north + z * sy];
    match p.kind() {
        Kind::Workbench => live_panel(
            mesh,
            g,
            p,
            [2., 2., 11., 4.],
            [
                point(2.4, 16.85, 14.20),
                point(13.2, 16.85, 14.20),
                point(13.2, 11.35, 14.20),
                point(2.4, 11.35, 14.20),
            ],
            [0., 0., 1.],
        ),
        Kind::ReceptionU => live_panel(
            mesh,
            g,
            p,
            [84., 2., 8., 4.],
            [
                point(82.8, 16.2, 20.02),
                point(93.2, 16.2, 20.02),
                point(93.2, 13.2, 20.02),
                point(82.8, 13.2, 20.02),
            ],
            [0., 0., 1.],
        ),
        Kind::MemoDesk | Kind::ReceptionL => {
            let (source_x, x, base) = if p.kind() == Kind::MemoDesk {
                (18., 17.2, 10.)
            } else {
                (34., 32.8, 12.)
            };
            live_panel(
                mesh,
                g,
                p,
                [source_x, 1., 4., 5.],
                [
                    point(x, base + 0.27, 2.3),
                    point(x + 4.5, base + 0.27, 2.3),
                    point(x + 4.5, base + 0.27, 7.1),
                    point(x, base + 0.27, 7.1),
                ],
                [0., 1., 0.],
            );
        }
        _ => {}
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
    include!("facility_radio_tests.rs");
}
