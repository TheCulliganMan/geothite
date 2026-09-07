//! Source-pixel building volume, culled to its exposed shell.
use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    geometry: &GridGeometry,
    placement: BuildingPlacement,
    inside: &[bool],
    luminance: &[u16],
    profile: &[usize],
    recessed: &[bool],
    body_left: usize,
    body_right: usize,
) {
    let width = placement.width * SOURCE_TILE_PIXELS;
    let height = placement.height * SOURCE_TILE_PIXELS;
    let roof_rows = placement.roof_rows * SOURCE_TILE_PIXELS;
    let wall = height - roof_rows;
    // The matched source plot supplies depth; its south edge is the door.
    let depth = height;
    let sx = geometry.tile_width / SOURCE_TILE_HEIGHT;
    let sy = geometry.tile_height / SOURCE_TILE_HEIGHT;
    let origin_x = geometry.origin_x + placement.column as f32 * geometry.tile_width;
    let origin_z = geometry.origin_z + placement.row as f32 * geometry.tile_height;
    let tops: Vec<_> = profile
        .iter()
        .map(|&p| (wall + 4).saturating_sub(p))
        .collect();
    let darkest = *luminance.iter().min().unwrap();
    let surface_rows: Vec<_> = profile
        .iter()
        .enumerate()
        .map(|(x, &top)| {
            (top..roof_rows)
                .find(|&y| inside[y * width + x] && luminance[y * width + x] > darkest)
                .unwrap_or(top.min(roof_rows - 1))
        })
        .collect();
    let occupied = |x: isize, y: isize, z: isize| {
        if x < 0 || y < 0 || z < 0 || x >= width as isize || z >= depth as isize {
            return false;
        }
        let (x, y, z) = (x as usize, y as usize, z as usize);
        if profile[x] >= roof_rows || y >= tops[x] {
            return false;
        }
        // The roof overrides the facade where its measured ends descend.
        if !(body_left..=body_right).contains(&x) && y + 4 < tops[x] {
            return false;
        }
        if z + 1 == depth && y < wall && y + 4 < tops[x] && recessed[(height - 1 - y) * width + x] {
            return false;
        }
        true
    };
    for x in 0..width {
        for y in 0..tops[x] {
            for z in 0..depth {
                if !occupied(x as isize, y as isize, z as isize) {
                    continue;
                }
                let x0 = origin_x + x as f32 * sx;
                let x1 = x0 + sx;
                let y0 = y as f32 * sy;
                let y1 = y0 + sy;
                let z0 = origin_z + z as f32 * sy;
                let z1 = z0 + sy;
                let faces = [
                    (
                        [0, 0, 1],
                        [[x1, y0, z1], [x1, y1, z1], [x0, y1, z1], [x0, y0, z1]],
                    ),
                    (
                        [0, 0, -1],
                        [[x0, y0, z0], [x0, y1, z0], [x1, y1, z0], [x1, y0, z0]],
                    ),
                    (
                        [-1, 0, 0],
                        [[x0, y0, z1], [x0, y1, z1], [x0, y1, z0], [x0, y0, z0]],
                    ),
                    (
                        [1, 0, 0],
                        [[x1, y0, z0], [x1, y1, z0], [x1, y1, z1], [x1, y0, z1]],
                    ),
                    (
                        [0, 1, 0],
                        [[x0, y1, z0], [x0, y1, z1], [x1, y1, z1], [x1, y1, z0]],
                    ),
                    (
                        [0, -1, 0],
                        [[x0, y0, z1], [x0, y0, z0], [x1, y0, z0], [x1, y0, z1]],
                    ),
                ];
                for (normal, corners) in faces {
                    if occupied(
                        x as isize + normal[0],
                        y as isize + normal[1],
                        z as isize + normal[2],
                    ) {
                        continue;
                    }
                    let roof = y + 4 >= tops[x];
                    let mut source_y = if roof {
                        roof_source_row(z, depth, roof_rows)
                            .max(surface_rows[x])
                            .min(roof_rows - 1)
                    } else {
                        height - 1 - y
                    };
                    if roof && normal[2] != 0 {
                        source_y = roof_rows.saturating_sub(tops[x] - y).min(roof_rows - 1);
                    }
                    let mut source_x = x;
                    if normal[0] != 0 && !roof {
                        source_x = facade_side_course_x(
                            inside,
                            luminance,
                            width,
                            source_y,
                            darkest,
                            normal[0] > 0,
                        );
                    }
                    if roof && normal[1] < 0 {
                        source_y = roof_rows - 1;
                    }
                    let shade = match normal {
                        [0, 0, 1] => 1.0,
                        [0, 1, 0] => 0.95,
                        [0, 0, -1] => 0.68,
                        [0, -1, 0] => 0.5,
                        _ => 0.78,
                    };
                    append_quad(
                        &mut mesh.textured,
                        corners,
                        normal.map(|n| n as f32),
                        source_pixel_uv(geometry, placement, source_x, source_y, true),
                        [shade, shade, shade, 1.0],
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silhouette_shell_has_closed_pixel_steps_and_a_recessed_front() {
        let geometry = GridGeometry {
            width: 1,
            height: 2,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let placement = BuildingPlacement {
            column: 0,
            row: 0,
            width: 1,
            height: 2,
            roof_rows: 1,
            ground_tile_index: 0,
        };
        let profile = [3, 2, 1, 0, 0, 1, 2, 3];
        let mut recess = vec![false; 128];
        recess[12 * 8 + 3] = true;
        let mut mesh = TerrainMeshData::default();
        append(
            &mut mesh,
            &geometry,
            placement,
            &[true; 128],
            &[100; 128],
            &profile,
            &recess,
            0,
            7,
        );
        let mut area_balance = Vec3::ZERO;
        for (quad, normal) in mesh
            .textured
            .positions
            .chunks_exact(4)
            .zip(mesh.textured.normals.chunks_exact(4))
        {
            let normal = Vec3::from_array(normal[0]);
            assert_eq!(
                normal.abs().element_sum(),
                1.0,
                "all faces are voxel planes"
            );
            let a = Vec3::from_array(quad[1]) - Vec3::from_array(quad[0]);
            let b = Vec3::from_array(quad[2]) - Vec3::from_array(quad[0]);
            assert!(a.cross(b).dot(normal) > 0.0, "outward winding");
            area_balance += normal * a.cross(b).length();
        }
        assert!(area_balance.length() < 0.001, "the shell must close");
        for (x, inset) in profile.into_iter().enumerate() {
            let peak = mesh
                .textured
                .positions
                .iter()
                .filter(|p| p[0] == x as f32)
                .map(|p| p[1])
                .fold(0.0, f32::max);
            // Shared edges may include the adjacent column's higher voxel.
            assert!(peak >= 12.0 - inset as f32);
        }
        assert!(
            mesh.textured
                .positions
                .chunks_exact(4)
                .zip(mesh.textured.normals.chunks_exact(4))
                .any(|(q, n)| n[0] == [0.0, 0.0, 1.0] && q[0] == [4.0, 3.0, 15.0]),
            "window is cut one voxel into the south facade"
        );
        assert!(
            mesh.textured
                .positions
                .iter()
                .all(|p| p[2] >= 0.0 && p[2] <= 16.0)
        );
    }
}
