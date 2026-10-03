//! Complete framed outdoor signs. Only native lettering remains a live surface.
//! A missing, rephased, overridden or already-owned cell rejects the whole sign.
use super::*;
use crate::live_profiles::{Document, Object};
use crate::outdoor_sign_models::{Kind, model};

#[derive(Clone, Copy, Debug)]
struct Binding {
    tileset: &'static str,
    block: u16,
    origin: [u8; 2],
    art: [u16; 4],
    backing: [u16; 4],
    kind: Kind,
    /// Pixel crop inside the original 16×16 drawing; never reconstructed text.
    lettering: [u8; 4],
}
include!("outdoor_sign_bindings.rs");

pub(super) struct Placement {
    binding: &'static Binding,
    column: usize,
    row: usize,
    ground: [usize; 4],
}
impl Placement {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        (0..4).map(move |i| (self.row + i / 2) * width + self.column + i % 2)
    }
}
fn coherent(cells: &[&VisualTile], g: &GridGeometry, x: usize, y: usize, b: &Binding) -> bool {
    cells.len() == g.width * g.height
        && x + 2 <= g.width
        && y + 2 <= g.height
        && (0..4).all(|i| {
            let s = &cells[(y + i / 2) * g.width + x + i % 2].source;
            s.tileset_id.as_ref() == b.tileset
                && s.metatile_id == b.block
                && s.subtile_column == b.origin[0] + (i % 2) as u8
                && s.subtile_row == b.origin[1] + (i / 2) as u8
                && s.tile_index == b.art[i]
        })
}
fn native_ground(map: &str, cell: &VisualTile, b: &Binding, tile: u16) -> bool {
    // Palette identity belongs to the native metatile as well as the atlas.
    // Every supported block contains its own verified backing vocabulary.
    cell.source.tileset_id.as_ref() == b.tileset
        && cell.source.metatile_id == b.block
        && cell.source.tile_index == tile
        && matches!(
            shape_for_source_on_map(map, &cell.source),
            CellShape::Flat | CellShape::PlaneAt { height: 0.0 }
        )
}
fn applies(map: &str, object: &Object) -> bool {
    object.map.as_deref().is_none_or(|m| m == map)
        && object
            .maps
            .as_ref()
            .is_none_or(|maps| maps.iter().any(|m| m == map))
}
fn profile_owns_source(map: &str, s: &VisualTileSource, document: Option<&Document>) -> bool {
    document.is_some_and(|d| {
        d.objects.iter().any(|o| {
            applies(map, o)
                && o.tileset == s.tileset_id.as_ref()
                && o.tiles.iter().enumerate().any(|(y, row)| {
                    row.iter().enumerate().any(|(x, &tile)| {
                        let sx = usize::from(o.origin[0]) + x;
                        let sy = usize::from(o.origin[1]) + y;
                        let block = o
                            .metatiles
                            .as_ref()
                            .and_then(|blocks| blocks.get(sy / 4))
                            .and_then(|row| row.get(sx / 4))
                            .copied()
                            .unwrap_or(o.metatile);
                        s.metatile_id == block
                            && usize::from(s.subtile_column) == sx % 4
                            && usize::from(s.subtile_row) == sy % 4
                            && s.tile_index == tile
                    })
                })
        })
    })
}
pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    profiles: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    if cells.len() != g.width * g.height
        || cells.len() != reserved.len()
        || g.width < 2
        || g.height < 2
    {
        return Vec::new();
    }
    let mut blocked = reserved.to_vec();
    let mut result = Vec::new();
    for b in BINDINGS {
        let mut ground = [0; 4];
        if !(0..4).all(|i| {
            if let Some(sample) = cells
                .iter()
                .position(|c| native_ground(map, c, b, b.backing[i]))
            {
                ground[i] = sample;
                true
            } else {
                false
            }
        }) {
            continue;
        }
        for y in 0..g.height - 1 {
            for x in 0..g.width - 1 {
                if !coherent(cells, g, x, y, b) {
                    continue;
                }
                let p = Placement {
                    binding: b,
                    column: x,
                    row: y,
                    ground,
                };
                // Source-level ownership protects incomplete/cropped custom
                // profiles too, even if their ground cannot be resolved. Check
                // only complete candidate signs, not every floor in the scene.
                // Existing Cinnabar profiles retain the same priority.
                if p.indices(g.width)
                    .any(|i| blocked[i] || profile_owns_source(map, &cells[i].source, profiles))
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
    let b = p.binding;
    if claimed.len() != cells.len()
        || !coherent(cells, g, p.column, p.row, b)
        || p.indices(g.width).any(|i| claimed[i])
        || !(0..4).all(|i| {
            p.ground[i] < cells.len() && native_ground("", cells[p.ground[i]], b, b.backing[i])
        })
    {
        return false;
    }
    for (local, index) in p.indices(g.width).enumerate() {
        let (w, e, n, s) = g.bounds(index % g.width, index / g.width);
        let sample = p.ground[local];
        append_top(
            &mut mesh.textured,
            [w, e, n, s],
            0.0,
            g.uv(sample % g.width, sample / g.width),
        );
    }
    let (w, _, _, s) = g.bounds(p.column, p.row + 1);
    // Feet and all weather caps remain within this exact 2×2 native plot.
    let fitted = [
        w + 0.06 * g.tile_width,
        w + 1.94 * g.tile_width,
        s - 0.62 * g.tile_height,
        s - 0.05 * g.tile_height,
    ];
    let height = b.kind.height() * g.tile_height;
    let m = model(b.kind);
    m.mesh.append_fitted(&mut mesh.solid, fitted, 0.0, height);
    let [left, right, bottom, top, front] = m.face;
    let fx = |v| fitted[0] + (fitted[1] - fitted[0]) * v;
    let z = fitted[2] + (fitted[3] - fitted[2]) * front;
    let [px, py, pw, ph] = b.lettering.map(f32::from);
    let u0 = (p.column as f32 + px / 8.) / g.width as f32;
    let u1 = (p.column as f32 + (px + pw) / 8.) / g.width as f32;
    let v0 = (p.row as f32 + py / 8.) / g.height as f32;
    let v1 = (p.row as f32 + (py + ph) / 8.) / g.height as f32;
    append_quad(
        &mut mesh.textured,
        [
            [fx(left), top * height, z],
            [fx(left), bottom * height, z],
            [fx(right), bottom * height, z],
            [fx(right), top * height, z],
        ],
        [0., 0., 1.],
        [[u0, v0], [u0, v1], [u1, v1], [u1, v0]],
        TEXTURED_SHADE,
    );
    for i in p.indices(g.width) {
        claimed[i] = true;
    }
    mark_authored_rect(mesh, g, [p.column, p.row, 2, 2], b.kind.label());
    true
}
#[cfg(test)]
#[path = "outdoor_sign_tests.rs"]
mod tests;
