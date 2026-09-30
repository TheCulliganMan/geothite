//! Authored complete rock drawings and tower approaches that the legacy audit
//! grouped under “building”. These are source identities, never collision
//! guesses. Physical support follows the exact pre-existing formation rule.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Structure {
    Mesa,
    Cave,
    Forecourt,
    Ice,
}
fn identify(cells: &[&VisualTile], g: &GridGeometry, p: BuildingPlacement) -> Option<Structure> {
    if !phases(cells, g, p) {
        return None;
    }
    let first = &cells[p.row * g.width + p.column].source;
    let shape = match first.tileset_id.as_ref() {
        "johto"
            if drawing(
                cells,
                g,
                p,
                "johto",
                &[&[0x6a, 0x70, 0x6b], &[0x6c, 0x72, 0x6d]],
                0,
            ) =>
        {
            Structure::Mesa
        }
        "johto" if drawing(cells, g, p, "johto", &[&[0x74, 0x75]], 0) => Structure::Forecourt,
        "kanto"
            if drawing(
                cells,
                g,
                p,
                "kanto",
                &[&[0x3e, 0x3f, 0x3f, 0x3b], &[0x24, 0x06, 0x57, 0x25]],
                0,
            ) =>
        {
            Structure::Cave
        }
        "ice_path"
            if p.height == 8
                && ice_path_plateau_origins(cells, g.width, g.height)
                    .contains(&(p.column, p.row, p.width)) =>
        {
            Structure::Ice
        }
        _ => return None,
    };
    // Full tile identities validate the actual art, not just a same-number
    // metatile that a mod may have repainted or a clipped viewport invented.
    (0..p.height)
        .all(|y| {
            (0..p.width).all(|x| {
                let s = &cells[(p.row + y) * g.width + p.column + x].source;
                expected(s.tileset_id.as_ref(), s.metatile_id).is_some_and(|a| {
                    s.tile_index == a[s.subtile_row as usize * 4 + s.subtile_column as usize]
                })
            })
        })
        .then_some(shape)
}
fn ground_sample(cells: &[&VisualTile], shapes: &[CellShape], kind: Structure) -> Option<usize> {
    let (tileset, tile) = match kind {
        Structure::Mesa | Structure::Forecourt => ("johto", 6),
        Structure::Cave => ("kanto", KANTO_GROUND_TILE_INDEX),
        Structure::Ice => ("ice_path", crate::ice_path::CAVE_GROUND_TILE),
    };
    ground(cells, shapes, tileset, tile, None)
}
pub(super) fn ready(
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    p: BuildingPlacement,
) -> bool {
    identify(cells, g, p).is_some_and(|k| ground_sample(cells, shapes, k).is_some())
}
fn ice_kind(id: u16) -> Option<Kind> {
    Some(match id {
        0x04 => Kind::IceShelfNorthwest,
        0x05 => Kind::IceShelfNorth,
        0x06 => Kind::IceShelfNortheast,
        0x09 => Kind::IceShelfInterior,
        0x0c => Kind::IceShelfSouthwest,
        0x0d => Kind::IceShelfSouth,
        0x0e => Kind::IceShelfSoutheast,
        0x10 => Kind::IceShelfNotchEast,
        0x11 => Kind::IceShelfNotchWest,
        0x12 => Kind::IceShelfStairRight,
        0x3a => Kind::IceShelfStairCorner,
        0x3e => Kind::IceShelfStairLeft,
        _ => return None,
    })
}
/// Rigid logical-grid fitting preserves corner/notch/stair topology. In
/// particular a half-depth southern cap may not be stretched across the whole
/// block merely because its physical mesh bounds are shorter than the plot.
fn append_grid_model(
    out: &mut SurfaceMeshData,
    kind: Kind,
    center: [f32; 2],
    base: f32,
    unit: [f32; 2],
) {
    let m = model(kind);
    let offset = out.positions.len() as u32;
    let light = Vec3::new(-0.35, 0.85, 0.4).normalize();
    for ((p, n), c) in m
        .surface
        .positions
        .iter()
        .zip(&m.surface.normals)
        .zip(&m.surface.colors)
    {
        let normal = (Vec3::from_array(*n) / Vec3::new(unit[0], unit[1], unit[1])).normalize();
        let shade = 0.62 + 0.38 * normal.dot(light).max(0.0);
        let mut color = *c;
        for channel in &mut color[..3] {
            *channel *= shade;
        }
        out.positions.push([
            center[0] + p[0] * unit[0],
            base + p[1] * unit[1],
            center[1] + p[2] * unit[1],
        ]);
        out.normals.push(normal.to_array());
        out.uvs.push([0.; 2]);
        out.colors.push(color);
    }
    out.indices
        .extend(m.surface.indices.iter().map(|i| i + offset));
}
pub(super) fn append_building(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    p: BuildingPlacement,
    claimed: &mut [bool],
) -> bool {
    let Some(kind) = identify(cells, g, p) else {
        return false;
    };
    let rect = [p.column, p.row, p.width, p.height];
    if !clear(claimed, g, rect) {
        return false;
    }
    let Some(sample) = ground_sample(cells, shapes, kind) else {
        return false;
    };
    let b = bounds(g, rect);
    let u = g.tile_height;
    // A flush authored deck/cap owns its datum. A coplanar underlay would
    // z-fight with its top, even though both preserve the same walking height.
    if kind == Structure::Forecourt {
        floor(
            mesh,
            cells,
            shapes,
            g,
            [p.column, p.row + 2, p.width, 2],
            sample,
            claimed,
        );
        for y in p.row..p.row + 2 {
            for x in p.column..p.column + p.width {
                claimed[y * g.width + x] = true;
            }
        }
    } else if kind == Structure::Ice && !is_complete_rock_formation(cells, g, p) {
        for y in p.row..p.row + p.height {
            for x in p.column..p.column + p.width {
                claimed[y * g.width + x] = true;
            }
        }
    } else {
        floor(mesh, cells, shapes, g, rect, sample, claimed);
    }
    let label = match kind {
        Structure::Mesa => {
            let m = model(Kind::HighlandMesa);
            m.append_fitted(&mut mesh.solid, b, m.min[1] * u, u, None);
            Kind::HighlandMesa.label()
        }
        Structure::Cave => {
            model(Kind::DiglettCave).append_fitted(
                &mut mesh.solid,
                b,
                model(Kind::DiglettCave).min[1] * u,
                u,
                Some(b[0] + 5. * g.tile_width),
            );
            Kind::DiglettCave.label()
        }
        Structure::Forecourt => {
            // Source platform occupies the northern two rows. The remaining
            // two rows are the native path, never a raised deck or a building.
            append_grid_model(
                &mut mesh.solid,
                Kind::TowerForecourt,
                [(b[0] + b[1]) / 2., b[2] + u],
                0.,
                [g.tile_width, u],
            );
            Kind::TowerForecourt.label()
        }
        Structure::Ice => {
            // Preserve legacy support exactly, including its narrow-island
            // zero datum. We intentionally do not amend collision or support.
            let base = if is_complete_rock_formation(cells, g, p) {
                0.
            } else {
                -2. * u
            };
            for by in 0..2 {
                for bx in 0..p.width / 4 {
                    let id = cells[(p.row + by * 4) * g.width + p.column + bx * 4]
                        .source
                        .metatile_id;
                    let model_kind = ice_kind(id).expect("validated shelf art");
                    append_grid_model(
                        &mut mesh.solid,
                        model_kind,
                        [
                            b[0] + (bx * 4 + 2) as f32 * g.tile_width,
                            b[2] + (by * 4 + 2) as f32 * u,
                        ],
                        base,
                        [g.tile_width, u],
                    );
                }
            }
            "world-exterior/connected_ice_shelf"
        }
    };
    // The old pixel-building branch applies this exact rule after rendering.
    // Moving that branch into authored art must not lower any player's feet.
    if is_complete_rock_formation(cells, g, p) {
        let h = crate::cave::MOUND_FACE_HEIGHT * u / SOURCE_TILE_HEIGHT;
        for y in p.row..p.row + p.roof_rows {
            for x in p.column..p.column + p.width {
                mesh.footing_heights[y * g.width + x] = h;
            }
        }
    }
    mark_authored_rect(mesh, g, rect, label);
    true
}

// Compact source signatures, solely for rejecting changed or incomplete art.
// No texture/palette pixels, map blocks, collision data, or content are shipped.
fn expected(tileset: &str, id: u16) -> Option<[u16; 16]> {
    Some(match (tileset, id) {
        ("johto", 0x6a) => [
            0x3c, 0x2b, 0x2c, 0x2c, 0x2b, 0x3b, 0x3c, 0x3c, 0x3b, 0x3b, 0x3c, 0x3c, 0x3b, 0x3b,
            0x3c, 0x3c,
        ],
        ("johto", 0x70) => [
            0x2c, 0x2c, 0x2c, 0x2c, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c,
            0x3c, 0x3c,
        ],
        ("johto", 0x6b) => [
            0x2c, 0x2c, 0x2d, 0x3c, 0x3c, 0x3c, 0x3d, 0x2d, 0x3c, 0x3c, 0x3d, 0x3d, 0x3c, 0x3c,
            0x3d, 0x3d,
        ],
        ("johto", 0x6c) => [
            0x3b, 0x3b, 0x3c, 0x3c, 0x3b, 0x3b, 0x3c, 0x3c, 0x3b, 0x4b, 0x4c, 0x4c, 0x4b, 0x4c,
            0x4c, 0x4c,
        ],
        ("johto", 0x72) => [
            0x3c, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c, 0x3c, 0x4c, 0x4c, 0x4c, 0x4c, 0x4c, 0x4c,
            0x4c, 0x4c,
        ],
        ("johto", 0x6d) => [
            0x3c, 0x3c, 0x3d, 0x3d, 0x3c, 0x3c, 0x3d, 0x3d, 0x4c, 0x4c, 0x4d, 0x3d, 0x4c, 0x4c,
            0x4c, 0x4d,
        ],
        ("johto", 0x74) => [
            0x3b, 0x06, 0x06, 0x06, 0x4b, 0x4c, 0x9a, 0x9a, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06,
            0x06, 0x06,
        ],
        ("johto", 0x75) => [
            0x06, 0x06, 0x06, 0x3d, 0x4c, 0x4c, 0x4c, 0x4d, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06,
            0x06, 0x06,
        ],
        ("kanto", 0x3e) => [
            0x11, 0x1e, 0x01, 0x01, 0x1e, 0x27, 0x11, 0x11, 0x27, 0x27, 0x11, 0x11, 0x27, 0x27,
            0x11, 0x11,
        ],
        ("kanto", 0x3f) => [
            0x01, 0x01, 0x01, 0x01, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
            0x11, 0x11,
        ],
        ("kanto", 0x3b) => [
            0x01, 0x01, 0x02, 0x11, 0x11, 0x11, 0x24, 0x02, 0x11, 0x11, 0x24, 0x24, 0x11, 0x11,
            0x24, 0x24,
        ],
        ("kanto", 0x24) => [
            0x27, 0x27, 0x11, 0x11, 0x27, 0x27, 0x11, 0x11, 0x27, 0x36, 0x37, 0x37, 0x36, 0x37,
            0x37, 0x37,
        ],
        ("kanto", 0x06) => [
            0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x48, 0x49, 0x37, 0x37, 0x58, 0x59,
            0x37, 0x37,
        ],
        ("kanto", 0x57) => [
            0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x37, 0x37, 0x37, 0x37, 0x37, 0x37,
            0x37, 0x37,
        ],
        ("kanto", 0x25) => [
            0x11, 0x11, 0x24, 0x24, 0x11, 0x11, 0x24, 0x24, 0x37, 0x37, 0x34, 0x24, 0x37, 0x37,
            0x37, 0x34,
        ],
        ("ice_path", 0x04) => [
            0x88, 0x89, 0x8a, 0x8b, 0x98, 0x99, 0x9a, 0x9a, 0xa8, 0xa9, 0x9a, 0x9b, 0xa8, 0xa9,
            0x9a, 0x9a,
        ],
        ("ice_path", 0x05) => [
            0x8a, 0x8b, 0x8a, 0x8b, 0x9a, 0x9a, 0x9a, 0x9a, 0x9a, 0x9b, 0x9a, 0x9a, 0xaa, 0xaa,
            0x9b, 0x9a,
        ],
        ("ice_path", 0x06) => [
            0x8a, 0x8b, 0x8c, 0x8d, 0x9a, 0x9a, 0x9c, 0x9d, 0x9a, 0x9a, 0xac, 0xad, 0x9a, 0x9a,
            0xac, 0xad,
        ],
        ("ice_path", 0x09) => [
            0xaa, 0xaa, 0xaa, 0x9a, 0xaa, 0x19, 0x19, 0x9a, 0xaa, 0x19, 0x19, 0x9b, 0x9a, 0xaa,
            0x9a, 0x19,
        ],
        ("ice_path", 0x0c) => [
            0xa8, 0xa9, 0x9a, 0x9b, 0xa8, 0xa9, 0x9a, 0x9a, 0xb8, 0xb9, 0xba, 0xba, 0xc8, 0xc9,
            0xca, 0xca,
        ],
        ("ice_path", 0x0d) => [
            0x9a, 0x9a, 0x9a, 0x9a, 0x9a, 0x9a, 0x9a, 0x9a, 0xba, 0xbb, 0xba, 0xba, 0xca, 0xcb,
            0xca, 0xca,
        ],
        ("ice_path", 0x0e) => [
            0x9b, 0x9a, 0xac, 0xad, 0x9a, 0x9a, 0xac, 0xad, 0xba, 0xbb, 0xbc, 0xbd, 0xca, 0xcb,
            0xcc, 0xcd,
        ],
        ("ice_path", 0x10) => [
            0x9a, 0x9a, 0x9a, 0x9a, 0x9a, 0x9a, 0x9b, 0x12, 0x9a, 0x9b, 0xa0, 0xa1, 0x9a, 0x12,
            0xb0, 0xb1,
        ],
        ("ice_path", 0x11) => [
            0x9a, 0x9a, 0x9a, 0x9a, 0x9b, 0xaa, 0x9a, 0x9a, 0xa2, 0xa3, 0xaa, 0x9a, 0xb2, 0xb3,
            0xaa, 0x9a,
        ],
        ("ice_path", 0x12) => [
            0x12, 0x9b, 0xaa, 0x12, 0x9b, 0xab, 0x9b, 0x9b, 0x80, 0x81, 0xae, 0xaf, 0x90, 0x91,
            0xbe, 0xbf,
        ],
        ("ice_path", 0x3a) => [
            0x9a, 0x9a, 0x9c, 0x9d, 0x9a, 0x9a, 0xac, 0xad, 0xae, 0xaf, 0xbc, 0xbd, 0xbe, 0xbf,
            0xcc, 0xcd,
        ],
        ("ice_path", 0x3e) => [
            0x9a, 0x9a, 0x9a, 0x9a, 0x9b, 0x9a, 0x9a, 0x9a, 0x80, 0x81, 0xb9, 0xba, 0x90, 0x91,
            0xc9, 0xca,
        ],
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    fn fixture(
        tileset: &str,
        rows: &[&[u16]],
    ) -> (Vec<VisualTile>, GridGeometry, BuildingPlacement) {
        let w = rows[0].len() * 4;
        let h = rows.len() * 4;
        let mut tiles = Vec::new();
        for y in 0..h + 1 {
            for x in 0..w {
                let (id, tile) = if y == h {
                    match tileset {
                        "johto" => (0x01, 0x06),
                        "kanto" => (0x31, 0x39),
                        _ => (0x02, crate::ice_path::CAVE_GROUND_TILE),
                    }
                } else {
                    let id = rows[y / 4][x / 4];
                    (id, expected(tileset, id).unwrap()[(y % 4) * 4 + x % 4])
                };
                tiles.push(VisualTile {
                    column: x as u32,
                    row: y as u32,
                    source: VisualTileSource {
                        tileset_id: Arc::from(tileset),
                        metatile_id: id,
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: tile,
                    },
                    texture: Handle::default(),
                    priority: false,
                });
            }
        }
        (
            tiles,
            GridGeometry {
                width: w,
                height: h + 1,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            },
            BuildingPlacement {
                column: 0,
                row: 0,
                width: w,
                height: h,
                roof_rows: if h == 4 { 2 } else { 6 },
                ground_tile_index: 0,
            },
        )
    }
    #[test]
    fn structural_source_groups_require_every_native_tile_and_phase() {
        for (t, rows, kind) in [
            (
                "johto",
                &[&[0x6a, 0x70, 0x6b][..], &[0x6c, 0x72, 0x6d][..]][..],
                Structure::Mesa,
            ),
            (
                "kanto",
                &[&[0x3e, 0x3f, 0x3f, 0x3b][..], &[0x24, 0x06, 0x57, 0x25][..]][..],
                Structure::Cave,
            ),
            ("johto", &[&[0x74, 0x75][..]][..], Structure::Forecourt),
            (
                "ice_path",
                &[&[0x04, 0x05, 0x06][..], &[0x0c, 0x12, 0x0e][..]][..],
                Structure::Ice,
            ),
        ] {
            let (mut tiles, g, p) = fixture(t, rows);
            assert_eq!(
                identify(&tiles.iter().collect::<Vec<_>>(), &g, p),
                Some(kind)
            );
            let original = tiles[13].source.clone();
            tiles[13].source.tile_index ^= 1;
            assert!(identify(&tiles.iter().collect::<Vec<_>>(), &g, p).is_none());
            tiles[13].source = original.clone();
            tiles[13].source.subtile_column ^= 1;
            assert!(identify(&tiles.iter().collect::<Vec<_>>(), &g, p).is_none());
            tiles[13].source = original;
            tiles[13].source.tileset_id = Arc::from("unrelated_mod");
            assert!(identify(&tiles.iter().collect::<Vec<_>>(), &g, p).is_none());
        }
    }
    #[test]
    fn shelf_stair_and_notch_parts_keep_native_handedness() {
        assert_eq!(ice_kind(0x12), Some(Kind::IceShelfStairRight));
        assert_eq!(ice_kind(0x3e), Some(Kind::IceShelfStairLeft));
        assert_eq!(ice_kind(0x3a), Some(Kind::IceShelfStairCorner));
        assert_eq!(ice_kind(0x10), Some(Kind::IceShelfNotchEast));
        assert_eq!(ice_kind(0x11), Some(Kind::IceShelfNotchWest));
        assert_eq!(ice_kind(0x13), None);
    }
    #[test]
    fn authored_structure_keeps_exact_legacy_footing_and_has_no_partial_claims() {
        for rows in [
            &[&[0x04, 0x06][..], &[0x10, 0x3a][..]][..],
            &[&[0x04, 0x05, 0x06][..], &[0x0c, 0x12, 0x0e][..]][..],
        ] {
            let (tiles, g, p) = fixture("ice_path", rows);
            let refs = tiles.iter().collect::<Vec<_>>();
            let mut m = TerrainMeshData::default();
            m.footing_heights = vec![0.; tiles.len()];
            m.authored_cells = vec![None; tiles.len()];
            let mut claimed = vec![false; tiles.len()];
            let shapes = vec![CellShape::Flat; tiles.len()];
            assert!(append_building(&mut m, &refs, &shapes, &g, p, &mut claimed));
            for y in 0..g.height {
                for x in 0..g.width {
                    let expected = if p.width >= 12 && y < p.roof_rows {
                        16.
                    } else {
                        0.
                    };
                    assert_eq!(m.footing_heights[y * g.width + x], expected);
                    assert_eq!(claimed[y * g.width + x], y < p.height);
                }
            }
            assert!(!m.solid.indices.is_empty());
            let count = m.solid.indices.len();
            assert!(!append_building(
                &mut m,
                &refs,
                &shapes,
                &g,
                p,
                &mut claimed
            ));
            assert_eq!(m.solid.indices.len(), count);
        }
    }
    #[test]
    fn source_cropping_or_unknown_shelf_variant_does_not_claim_geometry() {
        let (mut a, g, mut p) = fixture("ice_path", &[&[4, 5, 6], &[0xc, 0x12, 0xe]]);
        p.width -= 1;
        assert!(identify(&a.iter().collect::<Vec<_>>(), &g, p).is_none());
        p.width += 1;
        a[4 * g.width + 4].source.metatile_id = 0x13;
        assert!(identify(&a.iter().collect::<Vec<_>>(), &g, p).is_none());
    }
}
