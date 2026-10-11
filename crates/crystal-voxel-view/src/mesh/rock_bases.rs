//! Rounded native rock contact corners on verified level ground.
use super::*;
use bevy::prelude::{Vec2, Vec3Swizzles};

fn floor(cell: &VisualTile, shape: CellShape, g: &GridGeometry) -> bool {
    matches!(cell.source.tileset_id.as_ref(), "johto" | "johto_modern")
        && matches!(cell.source.tile_index, 0x05 | 0x06)
        && shape.surface_height(g.tile_height).abs() < 0.001
}

pub(super) fn round(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    origin: [i32; 2],
) {
    if !mesh.solid.uvs.iter().any(|uv| uv[0] < -9000.) {
        return;
    }
    let mut corners = Vec::new();
    let mut patches = Vec::new();
    for (i, cell) in cells.iter().enumerate() {
        let source = &cell.source;
        if !matches!(source.tileset_id.as_ref(), "johto" | "johto_modern")
            || source.metatile_id != 0x0a
        {
            continue;
        }
        let inward = match (source.subtile_column, source.subtile_row) {
            (0, 0) => Vec2::new(1., 1.),
            (3, 0) => Vec2::new(-1., 1.),
            (0, 3) => Vec2::new(1., -1.),
            (3, 3) => Vec2::new(-1., -1.),
            _ => continue,
        };
        let cx = i % g.width;
        let cz = i / g.width;
        let height = shapes[i].surface_height(g.tile_height);
        if height <= crate::profile::JUMP_LEDGE_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT {
            continue;
        }
        // The contact radius fits inside one source cell. Prove its local
        // 2x2 rock neighborhood; a cropped distant corner must not change this
        // already-visible contact or make it pop back to a square base.
        let complete = (0..2).all(|dz| {
            (0..2).all(|dx| {
                let x = cx as isize + dx * inward.x as isize;
                let z = cz as isize + dz * inward.y as isize;
                if x < 0 || z < 0 || x >= g.width as isize || z >= g.height as isize {
                    return false;
                }
                let ni = z as usize * g.width + x as usize;
                let s = &cells[ni].source;
                s.tileset_id == source.tileset_id
                    && s.metatile_id == 0x0a
                    && s.subtile_column as isize
                        == source.subtile_column as isize + dx * inward.x as isize
                    && s.subtile_row as isize
                        == source.subtile_row as isize + dz * inward.y as isize
                    && (shapes[ni].surface_height(g.tile_height) - height).abs() < 0.001
                    && mesh
                        .authored_cells
                        .get(ni)
                        .is_none_or(|claim| claim.is_none())
            })
        });
        if !complete {
            continue;
        }
        let nx = cx as isize - inward.x as isize;
        let nz = cz as isize - inward.y as isize;
        if nx < 0 || nz < 0 || nx >= g.width as isize || nz >= g.height as isize {
            continue;
        }
        let neighbors = [
            cz * g.width + nx as usize,
            nz as usize * g.width + cx,
            nz as usize * g.width + nx as usize,
        ];
        if !neighbors.iter().all(|&ni| {
            floor(cells[ni], shapes[ni], g)
                && mesh
                    .authored_cells
                    .get(ni)
                    .is_none_or(|claim| claim.is_none())
        }) {
            continue;
        }
        let (w, e, n, s) = g.bounds(cx, cz);
        let corner = Vec2::new(
            if inward.x > 0. { w } else { e },
            if inward.y > 0. { n } else { s },
        );
        // Stable corner variation breaks the uniform rounded-box outline
        // without extending beyond the floor patch or changing with a
        // renderer viewport scroll.
        let world_x = origin[0] + cx as i32 + if inward.x < 0. { 1 } else { 0 };
        let world_z = origin[1] + cz as i32 + if inward.y < 0. { 1 } else { 0 };
        let mut hash =
            (world_x as u32).wrapping_mul(0x9e3779b9) ^ (world_z as u32).wrapping_mul(0x85ebca6b);
        hash ^= hash >> 16;
        let radius = g.tile_width.min(g.tile_height) * (0.70 + 0.25 * (hash & 1023) as f32 / 1023.);
        corners.push((corner, inward, radius, height));
        patches.push((cz * g.width + cx, neighbors[2]));
    }
    if corners.is_empty() {
        return;
    }
    for (p, uv) in mesh.solid.positions.iter_mut().zip(&mesh.solid.uvs) {
        if uv[0] >= -9000. {
            continue;
        }
        let original = Vec3::from_array(*p);
        for &(corner, inward, radius, height) in &corners {
            if original.y < -0.01 || original.y > height + 0.01 {
                continue;
            }
            let local = (original.xz() - corner) * inward;
            if local.x < -g.tile_width * 0.1
                || local.y < -g.tile_height * 0.1
                || local.x > radius
                || local.y > radius
            {
                continue;
            }
            let center = corner + inward * radius;
            let radial = original.xz() - center;
            let length = radial.length();
            if length > radius {
                let rounded = center + radial * (radius / length);
                p[0] = rounded.x;
                p[2] = rounded.y;
            }
        }
    }
    new_bark::append_rock_floor_patches(&mut mesh.solid, map, cells, g, origin, &patches);
    outdoor_stone::smooth_normals(&mut mesh.solid);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (VisualWorldFrame, GridGeometry, TerrainMeshData) {
        let flat = super::super::tests::source_with_tile(0, 0, 0, 5);
        let mut sources = vec![flat; 36];
        for z in 0..4 {
            for x in 0..4 {
                sources[(z + 1) * 6 + x + 1] =
                    super::super::tests::source_with_tile(0x0a, x as u8, z as u8, 0x3c);
            }
        }
        let frame = super::super::tests::frame(6, 6, sources);
        let g = GridGeometry {
            width: 6,
            height: 6,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let h = shape_for_source(&frame.tiles[7].source).surface_height(8.);
        let mut mesh = TerrainMeshData::default();
        mesh.footing_heights = frame
            .tiles
            .iter()
            .map(|t| shape_for_source(&t.source).surface_height(8.))
            .collect();
        mesh.authored_cells = vec![None; 36];
        for (p, n) in [
            (
                [[8., h, 8.], [8., h, 40.], [40., h, 40.], [40., h, 8.]],
                Vec3::Y,
            ),
            (
                [[8., 0., 40.], [8., h, 40.], [8., h, 8.], [8., 0., 8.]],
                Vec3::NEG_X,
            ),
            (
                [[40., 0., 8.], [40., h, 8.], [40., h, 40.], [40., 0., 40.]],
                Vec3::X,
            ),
            (
                [[8., 0., 8.], [8., h, 8.], [40., h, 8.], [40., 0., 8.]],
                Vec3::NEG_Z,
            ),
            (
                [[40., 0., 40.], [40., h, 40.], [8., h, 40.], [8., 0., 40.]],
                Vec3::Z,
            ),
        ] {
            outdoor_stone::append(&mut mesh.solid, p, n.to_array(), &g, [0, 0]);
        }
        (frame, g, mesh)
    }
    #[test]
    fn rounded_contacts_share_wall_vertices_and_expose_a_closed_floor() {
        let (frame, g, mut mesh) = fixture();
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let before = mesh.solid.positions.clone();
        let footing = mesh.footing_heights.clone();
        round(&mut mesh, "Route26", &cells, &shapes, &g, [0, 0]);
        let corner: Vec<_> = before
            .iter()
            .zip(&mesh.solid.positions)
            .filter(|(p, _)| **p == [8., 0., 8.])
            .map(|(_, p)| *p)
            .collect();
        assert!(corner.len() >= 2);
        assert!(corner.iter().all(|p| *p == corner[0]));
        assert!(corner[0][0] > 9.5 && corner[0][2] > 9.5 && corner[0][1] == 0.);
        assert_eq!(footing, mesh.footing_heights);
        let point = Vec2::new(8.25, 8.25);
        let covered = mesh.solid.indices.chunks_exact(3).any(|tri| {
            let ids = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
            if ids
                .iter()
                .any(|&i| mesh.solid.uvs[i][0] < -9000. || mesh.solid.positions[i][1].abs() > 0.001)
            {
                return false;
            }
            let [a, b, c] = ids.map(|i| Vec3::from_array(mesh.solid.positions[i]).xz());
            let area = (b - a).perp_dot(c - a);
            if area.abs() < 0.0001 {
                return false;
            }
            [
                (b - point).perp_dot(c - point) / area,
                (c - point).perp_dot(a - point) / area,
                (a - point).perp_dot(b - point) / area,
            ]
            .iter()
            .all(|v| *v >= -0.0001)
        });
        assert!(
            covered,
            "ground must close the space revealed by rounding the contact corner"
        );
        for (original, after) in before.iter().zip(&mesh.solid.positions) {
            if original[0] == 24. && original[2] == 24. {
                assert_eq!(original, after);
            }
        }
    }
    #[test]
    fn contact_outline_stays_fixed_when_the_source_viewport_scrolls() {
        let (frame, g, mut mesh) = fixture();
        let original_count = mesh.solid.positions.len();
        let mut shifted = mesh.clone();
        for p in &mut shifted.solid.positions {
            p[0] += 8.;
            p[2] += 8.;
        }
        shifted.authored_cells = vec![None; 49];
        let flat = super::super::tests::source_with_tile(0, 0, 0, 5);
        let mut sources = vec![flat; 49];
        for z in 0..4 {
            for x in 0..4 {
                sources[(z + 2) * 7 + x + 2] =
                    super::super::tests::source_with_tile(0x0a, x as u8, z as u8, 0x3c);
            }
        }
        let shifted_frame = super::super::tests::frame(7, 7, sources);
        let shifted_g = GridGeometry {
            width: 7,
            height: 7,
            ..g
        };
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let other: Vec<_> = shifted_frame.tiles.iter().collect();
        let other_shapes: Vec<_> = other.iter().map(|c| shape_for_source(&c.source)).collect();
        round(&mut mesh, "Route26", &cells, &shapes, &g, [0, 0]);
        round(
            &mut shifted,
            "Route26",
            &other,
            &other_shapes,
            &shifted_g,
            [-1, -1],
        );
        for i in 0..original_count {
            let a = Vec3::from_array(mesh.solid.positions[i]);
            let b = Vec3::from_array(shifted.solid.positions[i]) - Vec3::new(8., 0., 8.);
            assert!(
                a.distance(b) < 0.0001,
                "viewport changes cannot reshape an existing rock contact"
            );
        }
    }
    #[test]
    fn a_ledge_neighbor_keeps_its_original_rock_contact() {
        let (frame, g, mut mesh) = fixture();
        let cells: Vec<_> = frame.tiles.iter().collect();
        let mut shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        shapes[6] = CellShape::RaisedTop {
            height: crate::profile::JUMP_LEDGE_HEIGHT,
            solid: SolidKind::Bank,
        };
        let before = mesh.solid.positions.clone();
        round(&mut mesh, "Route26", &cells, &shapes, &g, [0, 0]);
        for (original, after) in before.iter().zip(&mesh.solid.positions) {
            if *original == [8., 0., 8.] {
                assert_eq!(original, after);
            }
        }
        assert!(
            mesh.solid.positions.len() > before.len(),
            "other verified ground corners can still be rounded"
        );
    }
    #[test]
    fn a_distant_viewport_cut_preserves_the_visible_verified_corner() {
        let (_, g, mut mesh) = fixture();
        let flat = super::super::tests::source_with_tile(0, 0, 0, 5);
        let mut sources = vec![flat; 16];
        for z in 0..3 {
            for x in 0..3 {
                sources[(z + 1) * 4 + x + 1] =
                    super::super::tests::source_with_tile(0x0a, x as u8, z as u8, 0x3c);
            }
        }
        let frame = super::super::tests::frame(4, 4, sources);
        let g = GridGeometry {
            width: 4,
            height: 4,
            ..g
        };
        mesh.authored_cells = vec![None; 16];
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let original = mesh.solid.positions.clone();
        round(&mut mesh, "Route26", &cells, &shapes, &g, [0, 0]);
        let corners: Vec<_> = original
            .iter()
            .zip(&mesh.solid.positions)
            .filter(|(p, _)| **p == [8., 0., 8.])
            .map(|(_, p)| *p)
            .collect();
        assert!(corners.len() >= 2);
        assert!(corners
            .iter()
            .all(|p| p[0] > 9.5 && p[2] > 9.5 && p[1] == 0.));
    }
    #[test]
    fn altered_or_authored_corner_neighbors_do_not_infer_ground() {
        for authored in [false, true] {
            let (mut frame, g, mut mesh) = fixture();
            if authored {
                mesh.authored_cells[14] = Some("authored/rock");
            } else {
                frame.tiles[14].source = super::super::tests::source_with_tile(0, 0, 0, 5);
            }
            let cells: Vec<_> = frame.tiles.iter().collect();
            let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
            let before = mesh.solid.positions.clone();
            round(&mut mesh, "Route26", &cells, &shapes, &g, [0, 0]);
            for (original, after) in before.iter().zip(&mesh.solid.positions) {
                if *original == [8., 0., 8.] {
                    assert_eq!(original, after);
                }
            }
            assert!(
                mesh.solid.positions.len() > before.len(),
                "independent verified corners remain eligible"
            );
        }
    }
}
