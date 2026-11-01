//! Rounded native rock contact corners on verified level ground.
use super::*;
use bevy::prelude::{Vec2, Vec3Swizzles};

fn floor(cell: &VisualTile, shape: CellShape, g: &GridGeometry) -> bool {
    matches!(cell.source.tileset_id.as_ref(), "johto" | "johto_modern")
        && matches!(cell.source.tile_index, 0x05 | 0x06)
        && shape.surface_height(g.tile_height).abs() < 0.001
}

pub(super) fn round_geometry(
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
            if map == "BlackthornCity" {
                // The mountain town exposes broad fractured beds. A circular
                // contact projection bows every height ring into a padded
                // shoulder; one diagonal mineral plane keeps these rings
                // straight and shares the same cut across cap and wall.
                let cut=radius*(2.-std::f32::consts::SQRT_2);
                let shift=((cut-local.x-local.y)*0.5).max(0.);
                p[0]+=inward.x*shift;
                p[2]+=inward.y*shift;
            } else {
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
    }
    new_bark::append_rock_floor_patches(&mut mesh.solid, map, cells, g, origin, &patches);
}

#[cfg(test)]
fn round(mesh: &mut TerrainMeshData, map: &str, cells: &[&VisualTile],
    shapes: &[CellShape], g: &GridGeometry, origin: [i32; 2]) {
    round_geometry(mesh, map, cells, shapes, g, origin);
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
    fn production_rock_finish_keeps_fractures_after_rounding_the_contact_base() {
        for origin in [[0, 0], [-31, -7], [17, 43], [64, -32]] {
            check_production_rock_finish(origin);
        }
    }

    #[test]
    fn production_rock_finish_preserves_contacts_across_route_mineral_phases() {
        // Different routes sample different mineral beds even when they share
        // the same source drawing. Four favourable offsets cannot establish
        // that the combined rim/base deformation stays closed and outward.
        // Exercise negative coordinates too: connected-map views can cross
        // the local origin while retaining the same authored rock placement.
        for z in -3..=3 {
            for x in -3..=3 {
                check_production_rock_finish([x * 17, z * 19]);
            }
        }
    }

    #[test]
    fn indexed_rock_fields_match_the_full_scan_with_distant_source_masses() {
        let (original, mut g, mesh) = fixture();
        let width = 16;
        let flat = super::super::tests::source_with_tile(0, 0, 0, 5);
        let mut sources = vec![flat; width * g.height];
        for z in 0..g.height {
            for x in 0..g.width {
                sources[z * width + x] = original.tiles[z * g.width + x].source.clone();
                if (1..5).contains(&x) {
                    sources[z * width + x + 9] = original.tiles[z * g.width + x].source.clone();
                }
            }
        }
        g.width = width;
        let frame = super::super::tests::frame(width as u32, g.height as u32, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        for origin in [[0,0], [-31,-7], [17,43]] {
            let mut indexed = mesh.clone();
            indexed.footing_heights = shapes.iter().map(|s| s.surface_height(g.tile_height)).collect();
            indexed.authored_cells = vec![None; cells.len()];
            let mut reference = indexed.clone();
            outdoor_stone::sculpt_rock_rims(&mut indexed, &cells, &shapes, &g, origin);
            outdoor_stone::sculpt_rock_rims_reference(&mut reference, &cells, &shapes, &g, origin);
            assert_eq!(indexed, reference,
                "discarding distant zero-influence edges must preserve every output at {origin:?}");
        }
    }

    #[test]
    fn blackthorn_angular_contacts_keep_combined_mineral_winding_and_support() {
        for z in -3..=3 {for x in -3..=3 {
            check_production_rock_finish_for_map([x*17,z*19],"BlackthornCity");
        }}
    }

    fn check_production_rock_finish(origin: [i32; 2]) {
        check_production_rock_finish_for_map(origin,"Route26");
    }

    fn check_production_rock_finish_for_map(origin: [i32; 2], map: &str) {
        let (frame, g, mut mesh) = fixture();
        // Match production's six subdivisions per source tile on both caps
        // and walls, rather than six across the entire diagnostic shelf.
        let h = mesh.footing_heights[7];
        mesh.solid = SurfaceMeshData::default();
        for row in 1..5 { for column in 1..5 {
            let (w, e, n, south) = g.bounds(column, row);
            outdoor_stone::append(&mut mesh.solid,
                [[w, h, n], [w, h, south], [e, h, south], [e, h, n]],
                Vec3::Y.to_array(), &g, [0, 0]);
            for (exposed, points, normal) in [
                (column == 1, [[w,0.,south],[w,h,south],[w,h,n],[w,0.,n]], Vec3::NEG_X),
                (column == 4, [[e,0.,n],[e,h,n],[e,h,south],[e,0.,south]], Vec3::X),
                (row == 1, [[w,0.,n],[w,h,n],[e,h,n],[e,0.,n]], Vec3::NEG_Z),
                (row == 4, [[e,0.,south],[e,h,south],[w,h,south],[w,0.,south]], Vec3::Z),
            ] {
                if exposed {
                    outdoor_stone::append(&mut mesh.solid, points, normal.to_array(), &g, [0,0]);
                }
            }
        }}
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let corner = mesh.solid.positions.iter().position(|p| *p == [8., 0., 8.]).unwrap();
        let angle = 50_f32.to_radians();
        let center = Vec3::new(200., 48., 200.);
        let side = Vec3::new(angle.cos(), angle.sin(), 0.) * 4.;
        let first = mesh.solid.positions.len();
        for points in [[center, center - Vec3::X * 4., center + Vec3::Z * 4.],
            [center, center + Vec3::Z * 4., center + side]] {
            let start = mesh.solid.positions.len() as u32;
            mesh.solid.positions.extend(points.map(|p| p.to_array()));
            mesh.solid.normals.extend([Vec3::Y.to_array(); 3]);
            mesh.solid.colors.extend([[0.2, 0.2, 0.2, 1.]; 3]);
            mesh.solid.uvs.extend(points.map(|p| [-10000. - p.x, -20000. - p.z]));
            mesh.solid.indices.extend([start, start + 1, start + 2]);
        }
        let footing = mesh.footing_heights.clone();
        let claims = mesh.authored_cells.clone();
        for uv in &mut mesh.solid.uvs {
            if uv[0] < -9000. {
                uv[0] -= origin[0] as f32 * g.tile_width;
                uv[1] -= origin[1] as f32 * g.tile_height;
            }
        }
        // UVs encode X/Z on walls too: weathered vertical faces can have a
        // positive top projection without being caps. Retain the authored
        // face classification before the final normal pass, including the
        // corner caps and walls added by the sculptor.
        let source_positions = mesh.solid.positions.clone();
        let mut classified = mesh.clone();
        outdoor_stone::sculpt_rock_rims_on_map(&mut classified, map, &cells, &shapes, &g, origin);
        let mut reference = mesh.clone();
        outdoor_stone::sculpt_rock_rims_reference_on_map(&mut reference, map, &cells, &shapes, &g, origin);
        assert_eq!(classified.solid, reference.solid,
            "local edge candidates must preserve the full mineral field exactly at {origin:?}");
        let sculpted_positions = classified.solid.positions.clone();
        outdoor_stone::retain_mineral_cap_ownership(&mut classified);
        round_geometry(&mut classified, map, &cells, &shapes, &g, origin);
        super::super::outdoor_mountains::finish(&mut classified, map, &cells, &shapes, &g, origin,None);
        super::super::outdoor_mountains::append_reveal_floor(&mut classified, map, &cells, &shapes, &g, origin);
        outdoor_stone::finish_rock_geometry(&mut mesh, map, &cells, &shapes, &g, origin,None);
        assert_eq!(mesh.solid.positions, classified.solid.positions);
        assert_eq!(mesh.solid.indices, classified.solid.indices);
        let mut shared_source_positions = HashMap::new();
        for (i, source_position) in source_positions.iter().enumerate() {
            if mesh.solid.uvs[i][0] >= -9000. || source_position[0] > 100.
                || sculpted_positions[i] == *source_position {
                continue;
            }
            // Independent cap/wall interpolation can differ by one float ULP.
            // Those shared authored samples must not become a visible crack
            // after the steep shoulder deformation amplifies the difference.
            let key = source_position.map(|value| (value * 10000.).round() as i64);
            let changed = [0, 1, 2].map(|axis| sculpted_positions[i][axis] != source_position[axis]);
            if let Some((previous, previous_changed)) = shared_source_positions.insert(key, (mesh.solid.positions[i], changed)) {
                for axis in 0..3 {
                    if changed[axis] && previous_changed[axis] {
                        assert_eq!(previous[axis], mesh.solid.positions[i][axis],
                            "coincident source samples must retain one deformed coordinate at {origin:?}: {source_position:?}, axis {axis}");
                    }
                }
            }
        }
        assert!(mesh.solid.positions[corner][0] > 9.5);
        assert_eq!(mesh.footing_heights, footing);
        assert_eq!(mesh.authored_cells, claims);
        let faces = [first, first + 3].map(|start| {
            let p = [start, start + 1, start + 2]
                .map(|i| Vec3::from_array(mesh.solid.positions[i]));
            (p[1] - p[0]).cross(p[2] - p[0]).normalize()
        });
        // World-phase erosion changes the injected planes' final angle too.
        // Preserve fractures beyond the real crease threshold; shallow joins
        // should instead share a normal, rather than keep an arbitrary 50°
        // source-space separation after the geometry has actually changed.
        if faces[0].dot(faces[1]) < 25_f32.to_radians().cos() {
            for (start, face) in [first, first + 3].into_iter().zip(faces) {
                assert!(Vec3::from_array(mesh.solid.normals[start]).dot(face) > 0.99999,
                    "the production base pass must not erase the cap's fracture plane at {origin:?}");
            }
        } else {
            assert_eq!(mesh.solid.normals[first], mesh.solid.normals[first + 3],
                "shallow deformed planes must share their production normal at {origin:?}");
        }
        let mut caps = 0;
        let mut walls = 0;
        for ids in mesh.solid.indices.chunks_exact(3) {
            let ids = [ids[0] as usize, ids[1] as usize, ids[2] as usize];
            if mesh.solid.uvs[ids[0]][0] >= -9000. {
                continue;
            }
            let p = ids.map(|i| Vec3::from_array(mesh.solid.positions[i]));
            let normal = Vec3::from_array(classified.solid.normals[ids[0]]);
            let cross = (p[1] - p[0]).cross(p[2] - p[0]);
            let sculpted = ids.map(|i| Vec3::from_array(sculpted_positions[i]));
            let cap = normal.y > 0.9;
            // A rounded corner legitimately rotates a wall away from its
            // original cardinal normal. This convex fixture's interior is
            // at (24,24); test walls against that interior, caps against Y.
            let center = (p[0] + p[1] + p[2]) / 3.;
            let outward = if cap { Vec3::Y } else {
                Vec3::new(center.x - 24., 0., center.z - 24.).normalize()
            };
            assert!(cross.dot(outward) >= -0.0001,
                "source phase {origin:?} must retain outward winding: {p:?}; cap: {cap}; before base rounding: {sculpted:?}");
            if cap {
                caps += 1;
            } else {
                walls += 1;
            }
        }
        assert!(caps > 60);
        assert!(walls > 60, "restored walls must be checked as well as caps");
    }

    #[test]
    fn rounded_contacts_share_wall_vertices_and_expose_a_closed_floor() {
        for map in ["Route26", "BlackthornCity"] {
        let (frame, g, mut mesh) = fixture();
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let before = mesh.solid.positions.clone();
        let footing = mesh.footing_heights.clone();
        round(&mut mesh, map, &cells, &shapes, &g, [0, 0]);
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
    }
    #[test]
    fn blackthorn_contact_samples_form_one_straight_mineral_plane() {
        let (frame,g,mut mesh)=fixture();
        let original=mesh.solid.positions.clone();
        let cells:Vec<_>=frame.tiles.iter().collect();
        let shapes:Vec<_>=cells.iter().map(|c|shape_for_source(&c.source)).collect();
        round_geometry(&mut mesh,"BlackthornCity",&cells,&shapes,&g,[0,0]);
        let mut cut=original.iter().zip(&mesh.solid.positions).filter_map(|(p,q)| {
            (p[0]==8. && p[1]==0. && p[2]>=8. && p[2]<=11. && q[0]>8.0001)
                .then_some(Vec2::new(q[0],q[2]))
        }).collect::<Vec<_>>();
        cut.sort_by(|a,b|a.x.total_cmp(&b.x));
        cut.dedup();
        assert!(cut.len()>=3,"inspect more than the single corner endpoint");
        let axis=(cut[cut.len()-1]-cut[0]).normalize();
        assert!(cut.iter().all(|p|axis.perp_dot(*p-cut[0]).abs()<0.0001),
            "contact rings must lie on a straight cut rather than a circular arc");
    }

    #[test]
    fn contact_outline_stays_fixed_when_the_source_viewport_scrolls() {
        for map in ["Route26","BlackthornCity"] {
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
        round(&mut mesh, map, &cells, &shapes, &g, [0, 0]);
        round(
            &mut shifted,
            map,
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
