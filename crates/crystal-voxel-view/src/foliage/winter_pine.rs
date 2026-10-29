//! Snow settles on upward-facing needles while the bark stays warm brown.
use crate::mesh::TerrainMeshData;

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    bounds: [f32; 4],
    base: f32,
    tile: f32,
    coordinates: [i32; 2],
    neighbors: [bool; 4],
) {
    let first = mesh.solid.positions.len();
    super::pine::append_canopy(mesh, bounds, base, tile, coordinates, neighbors);
    for index in first..mesh.solid.colors.len() {
        let color = &mut mesh.solid.colors[index];
        let upward = mesh.solid.normals[index][1];
        if color[1] > color[0] * 1.15 && color[1] > color[2] * 1.05 && upward > 0.12 {
            let coverage = ((upward - 0.12) * 2.5).clamp(0.0, 0.94);
            for (channel, snow) in [0.82, 0.91, 0.95].into_iter().enumerate() {
                color[channel] += (snow - color[channel]) * coverage;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::append;
    use crate::mesh::TerrainMeshData;

    #[test]
    fn snow_preserves_pine_geometry_and_bark() {
        let mut native = TerrainMeshData::default();
        super::super::pine::append_canopy(
            &mut native,
            [0.0, 16.0, 0.0, 16.0],
            0.0,
            4.0,
            [12, 8],
            [false; 4],
        );
        let mut winter = TerrainMeshData::default();
        append(
            &mut winter,
            [0.0, 16.0, 0.0, 16.0],
            0.0,
            4.0,
            [12, 8],
            [false; 4],
        );
        assert_eq!(winter.solid.positions, native.solid.positions);
        assert_eq!(winter.solid.normals, native.solid.normals);
        assert_eq!(winter.solid.indices, native.solid.indices);
        let mut snowy = 0;
        for (original, painted) in native.solid.colors.iter().zip(&winter.solid.colors) {
            if original[0] > original[1] {
                assert_eq!(
                    original, painted,
                    "brown bark must retain its native pigment"
                );
            }
            if painted[0] > 0.5 && painted[2] > 0.6 {
                snowy += 1;
            }
        }
        assert!(
            snowy > 100,
            "snow must cover real upward-facing needle geometry"
        );
    }
}
