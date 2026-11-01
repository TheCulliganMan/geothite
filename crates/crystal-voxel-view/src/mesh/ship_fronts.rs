//! Original closed room-front shells with canonical plain and porthole faces.
//! Only source-complete FastShipB1F fronts may replace the old flat cap art.
use super::*;
use crate::interior_models::Model;
use crate::live_profiles::Document;
use std::sync::OnceLock;
#[path = "ship_front_source.rs"]
mod source;
use source::{Identity, Kind};

pub(super) struct Placement {
    resolved: source::Match,
}
impl Placement {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> {
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
fn shell(kind: Kind) -> &'static Model {
    static MODELS: OnceLock<[Model; 2]> = OnceLock::new();
    &MODELS.get_or_init(|| {
        [
            crate::model_storage::include_model!(
                "models/ship_fronts/room_front_west.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/ship_fronts/room_front_east.mesh.json"
            ),
        ]
        .map(|s| Model::parse(s).expect("validated original room-front shell"))
    })[kind as usize]
}
fn bundled_profiles() -> &'static Document {
    static DOCUMENT: OnceLock<Document> = OnceLock::new();
    DOCUMENT.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../../modpacks/voxel-view/profiles.json"
        ))
        .expect("validated bundled ship wall profiles")
    })
}
fn canonical_replaced_profile(object: &crate::live_profiles::Object) -> bool {
    matches!(
        object.name.as_str(),
        "Shared ship interior partition panel"
            | "Shared ship partition end 1e"
            | "Shared ship partition end 17"
            | "Shared ship partition end 1c"
            | "Shared ship partition end 1f"
    ) && bundled_profiles()
        .objects
        .iter()
        .any(|canonical| canonical == object)
}
pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    if map != "FastShipB1F"
        || g.width.checked_mul(g.height) != Some(cells.len())
        || reserved.len() != cells.len()
    {
        return Vec::new();
    }
    let mut blocked = reserved.to_vec();
    // Replace only the five exact unchanged bundled flat partition profiles.
    // Filter before resolving: a canonical object must never mask a later
    // overlapping custom object and thereby steal its source ownership.
    let customized = profiles.map(|document| Document {
        objects: document
            .objects
            .iter()
            .filter(|object| !canonical_replaced_profile(object))
            .cloned()
            .collect(),
        atmosphere: None,
    });
    for p in live::resolve(cells, g.width, g.height, map, customized.as_ref()) {
        for i in p.indices(g.width) {
            blocked[i] = true;
        }
    }
    let identities: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    source::resolve(map, &identities, g.width, g.height, origin, &blocked)
        .into_iter()
        .map(|resolved| Placement { resolved })
        .collect()
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    placement: &Placement,
    claimed: &mut [bool],
) -> bool {
    let p = placement.resolved;
    if claimed.len() != cells.len() {
        return false;
    }
    let identities: Vec<_> = cells.iter().map(|c| identity(&c.source)).collect();
    if !p.coherent(&identities, g.width, g.height) || p.guard_indices(g.width).any(|i| claimed[i]) {
        return false;
    }
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let sx = g.tile_width / 8.;
    let sz = g.tile_height / 8.;
    // The same quiet linoleum used by the adjoining room remains at zero
    // under the wall reveal. It is emitted before the wall's cutaway range.
    for i in p.indices(g.width) {
        modeled_interiors::append_ship_front_backing(
            &mut mesh.solid,
            g.bounds(i % g.width, i / g.width).into(),
            [
                p.origin[0] + (i % g.width) as i32,
                p.origin[1] + (i / g.width) as i32,
            ],
        );
    }
    let start = mesh.solid.positions.len();
    shell(p.kind).append_fitted(
        &mut mesh.solid,
        [w, w + 192. * sx, n, n + 32. * sz],
        0.,
        crate::ship::B1F_VISUAL_WALL_HEIGHT * sz,
    );
    // Full models retain every reverse and side face. No camera-dependent
    // culling is baked into the asset. Source 02/03 gets the existing porthole;
    // the other face/jamb courses get the existing closed plain steel panel.
    let entrance = p.kind.entrance();
    for (a, b) in [(2, entrance), (entrance + 4, 22)] {
        let mut x = a;
        while x < b {
            let s = &cells[(p.row + 2) * g.width + p.column + x].source;
            let porthole = s.tile_index == 0x02;
            let length = if porthole { 2 } else { 1 };
            let bounds = [
                w + x as f32 * g.tile_width,
                w + (x + length) as f32 * g.tile_width,
                n + 29. * sz,
                n + 32. * sz,
            ];
            if porthole {
                crate::dungeon_models::model(crate::dungeon_models::Kind::PortholeBulkhead)
                    .append_porthole(
                        &mut mesh.solid,
                        bounds,
                        0.,
                        16. * sz,
                        crate::ship::B1F_VISUAL_WALL_HEIGHT * sz,
                    );
            } else {
                crate::dungeon_models::extension_model(
                    crate::dungeon_models::ExtensionKind::ShipBulkhead,
                )
                .append(
                    &mut mesh.solid,
                    bounds,
                    0.,
                    crate::ship::B1F_VISUAL_WALL_HEIGHT * sz,
                );
            }
            x += length;
        }
    }
    mesh.solid
        .cutaway_ranges
        .push(start..mesh.solid.positions.len());
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
    include!("ship_front_tests.rs");
}
