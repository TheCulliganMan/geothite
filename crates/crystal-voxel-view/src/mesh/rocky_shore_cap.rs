//! Low bedrock crowns inside authenticated Kanto bank cells.
use bevy::math::Vec3;

use super::{GridGeometry, SurfaceMeshData};

/// Preserve every native contact at ground height. Only the interior of the
/// blocking bank gains relief; paths, water and actor support stay unchanged.
pub(super) fn append(
    mesh: &mut SurfaceMeshData,
    corners: [[f32; 3]; 4],
    grid: &GridGeometry,
    origin: [i32; 2],
    palette: [[f32; 3]; 2],
) {
    let palette = palette.map(|color| {
        let light = Vec3::from_array(color).dot(Vec3::new(0.2126, 0.7152, 0.0722));
        std::array::from_fn(|c| (color[c] * 0.12 + light * [1.06, 1.02, 0.94][c] * 0.88) * 0.72)
    });
    let first = mesh.positions.len();
    super::new_bark::ledge_finish::append_shore(mesh, corners, [0., 1., 0.], grid, origin, palette);
    // Retain the current source exposure while using the common bedrock grain.
    for (position, uv) in mesh.positions[first..].iter().zip(&mut mesh.uvs[first..]) {
        *uv = [
            88000. + (position[0] - grid.origin_x + origin[0] as f32 * grid.tile_width) / 8.,
            -70000. - (position[2] - grid.origin_z + origin[1] as f32 * grid.tile_height) / 8.,
        ];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crown_keeps_contacts_and_winding_and_scrolls_with_the_native_bank() {
        let grid = GridGeometry {
            width: 1,
            height: 1,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let corners = [[0., 0., 0.], [0., 0., 8.], [8., 0., 8.], [8., 0., 0.]];
        let palette = [[0.4, 0.35, 0.25], [0.2, 0.18, 0.13]];
        let mut a = SurfaceMeshData::default();
        append(&mut a, corners, &grid, [3, 5], palette);
        assert!(corners.iter().all(|p| a.positions.contains(p)));
        assert!(a.positions.iter().all(|p| p[0] >= 0.
            && p[0] <= 8.
            && p[2] >= 0.
            && p[2] <= 8.
            && p[1] >= 0.
            && p[1] < 1.));
        assert!(a.normals.iter().all(|n| n[1] > 0.9));
        let moved = GridGeometry {
            origin_x: 16.,
            origin_z: -24.,
            ..grid
        };
        let mut b = SurfaceMeshData::default();
        append(
            &mut b,
            corners.map(|p| [p[0] + 16., p[1], p[2] - 24.]),
            &moved,
            [3, 5],
            palette,
        );
        assert_eq!(a.uvs, b.uvs);
        assert_eq!(a.colors, b.colors);
        for (a, b) in a.positions.iter().zip(&b.positions) {
            assert!((a[1] - b[1]).abs() < 0.00001);
        }
    }
}
