//! The two Rocket B1F palms reuse the original radial-frond plant mesh/cache.
use super::*;
use crate::interior_models::{ModelKind, model};
use crate::live_profiles::Document;
#[path = "rocket_plants_source.rs"]
mod source;

pub(super) struct Placement(source::Match);
impl Placement {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> {
        self.0.indices(width)
    }
}
fn identity(s: &VisualTileSource) -> source::Identity<'_> {
    source::Identity {
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
    if map != source::MAP
        || g.width.checked_mul(g.height) != Some(cells.len())
        || reserved.len() != cells.len()
    {
        return Vec::new();
    }
    let blocked: Vec<_> = cells
        .iter()
        .enumerate()
        .map(|(i, cell)| {
            reserved[i] || native_ground_bindings::profile_owns(map, &cell.source, profiles)
        })
        .collect();
    let identities: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    source::resolve(map, &identities, g.width, g.height, origin, &blocked)
        .into_iter()
        .map(Placement)
        .collect()
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    placement: &Placement,
    claimed: &mut [bool],
) -> bool {
    let p = placement.0;
    if claimed.len() != cells.len() {
        return false;
    }
    let identities: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    if !p.coherent(&identities, g.width, g.height) || p.guard_indices(g.width).any(|i| claimed[i]) {
        return false;
    }
    let ground = p.ground(g.width);
    for i in p.indices(g.width) {
        append_top(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            0.,
            g.uv(ground % g.width, ground / g.width),
        );
    }
    let (west, _, north, _) = g.bounds(p.plant_column(), p.row);
    let sx = g.tile_width / 8.;
    let sy = g.tile_height / 8.;
    // Preserve the source 16px width, 24px foot line, and normal floor datum.
    // Radial drooping fronds, a visible trunk and rolled round pot match the
    // native silhouette; no generic bush or borrowed stretched furnishing.
    model(ModelKind::PlantTropic).append_fitted(
        &mut mesh.solid,
        [
            west + 0.36 * sx,
            west + 15.64 * sx,
            north + 12. * sy,
            north + 24. * sy,
        ],
        0.,
        23. * sy,
    );
    for i in p.indices(g.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some("interior/plant_tropic");
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    include!("rocket_plants_tests.rs");
}
