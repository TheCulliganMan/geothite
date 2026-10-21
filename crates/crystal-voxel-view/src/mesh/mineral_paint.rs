//! Register mineral detail on the completed three-dimensional surface.
use super::*;
use bevy::prelude::Vec2;

pub(super) fn finish(
    surface: &mut SurfaceMeshData,
    g: &GridGeometry,
    origin: [i32; 2],
    regions: &[crystal_render_api::VisualSourceRegion],
) {
    for (position, uv) in surface.positions.iter().zip(&mut surface.uvs) {
        if uv[0] >= -9000. {
            continue;
        }
        let source = super::mineral_coords::point(*position, *uv, g, origin, regions);
        let target = Vec2::new(
            g.origin_x + source.x * g.tile_width,
            g.origin_z + source.y * g.tile_height,
        );
        let placed = Vec2::new(position[0], position[2]);
        let delta = placed - target;
        // Repeated objects can retain distant donor paint. That is not proof
        // of a local geological deformation and must not relocate their art.
        if delta.length() > g.tile_width.max(g.tile_height) * 4. {
            continue;
        }
        // All geometry fitting has finished. The same native material and
        // source-map phase now follow the real slope in each projection.
        uv[0] -= delta.x;
        uv[1] -= delta.y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{IVec2, UVec2, Vec3};

    fn fixture(g: &GridGeometry, origin: [i32; 2], tag: f32) -> SurfaceMeshData {
        let source = [
            Vec3::new(2., 0., 3.),
            Vec3::new(2., 2., 3.),
            Vec3::new(3., 2., 3.),
        ];
        let mut mesh = SurfaceMeshData::default();
        mesh.positions = source
            .map(|p| {
                [
                    g.origin_x + p.x * g.tile_width + p.y * g.tile_width * 0.35,
                    p.y * g.tile_height,
                    g.origin_z + p.z * g.tile_height - p.y * g.tile_height * 0.40,
                ]
            })
            .to_vec();
        mesh.uvs = source
            .map(|p| {
                [
                    -10000. - (p.x + origin[0] as f32) * g.tile_width,
                    -tag - (p.z + origin[1] as f32) * g.tile_height,
                ]
            })
            .to_vec();
        mesh.normals = vec![[0., 0., 1.]; 3];
        mesh.colors = vec![[0.34, 0.22, 0.11, 1.]; 3];
        mesh.indices = vec![0, 1, 2];
        mesh
    }

    #[test]
    fn sculpted_stone_paint_covers_the_real_slope_without_changing_pigment_or_geometry() {
        for size in [8., 32., 64.] {
            for tag in [20000., 40000., 60000.] {
                let g = GridGeometry {
                    width: 12,
                    height: 12,
                    tile_width: size,
                    tile_height: size,
                    origin_x: -size * 6.,
                    origin_z: -size * 6.,
                };
                let origin = [20, 10];
                let mut mesh = fixture(&g, origin, tag);
                let original = mesh.clone();
                finish(&mut mesh, &g, origin, &[]);
                for (p, uv) in mesh.positions.iter().zip(&mesh.uvs) {
                    let actual = Vec2::new(p[0] - g.origin_x, p[2] - g.origin_z)
                        + Vec2::new(origin[0] as f32 * size, origin[1] as f32 * size);
                    assert!((uv[0] - (-10000. - actual.x)).abs() < 0.01);
                    assert!((uv[1] - (-tag - actual.y)).abs() < 0.01);
                }
                assert_eq!(mesh.positions, original.positions);
                assert_eq!(mesh.normals, original.normals);
                assert_eq!(mesh.colors, original.colors);
                assert_eq!(mesh.indices, original.indices);
                assert_ne!(
                    mesh.uvs[1][1], original.uvs[1][1],
                    "a sloped wall must not repeat one flat-coordinate row"
                );
            }
        }
    }

    #[test]
    fn projected_detail_keeps_its_source_phase_through_scrolls_and_connected_paint() {
        let g = GridGeometry {
            width: 12,
            height: 12,
            tile_width: 32.,
            tile_height: 32.,
            origin_x: -192.,
            origin_z: -192.,
        };
        let origin = [20, 10];
        let mut a = fixture(&g, origin, 40000.);
        let mut b = a.clone();
        let scroll = GridGeometry {
            origin_x: g.origin_x + 32.,
            origin_z: g.origin_z + 32.,
            ..g
        };
        finish(&mut a, &g, origin, &[]);
        finish(&mut b, &scroll, [21, 11], &[]);
        assert_eq!(
            a, b,
            "source paint must stay fixed when the terrain rectangle shifts"
        );
        let mut active = fixture(&g, origin, 40000.);
        let mut connected = active.clone();
        let shift = IVec2::new(16, -8);
        for uv in &mut connected.uvs {
            uv[0] += shift.x as f32 * 32.;
            uv[1] += shift.y as f32 * 32.;
        }
        let region = [crystal_render_api::VisualSourceRegion {
            map_id: "neighbor".into(),
            origin: shift,
            size: UVec2::splat(80),
        }];
        finish(&mut active, &g, origin, &region);
        finish(&mut connected, &g, origin, &region);
        for (a, b) in active.uvs.iter().zip(&connected.uvs) {
            assert!((b[0] - a[0] - shift.x as f32 * 32.).abs() < 0.001);
            assert!((b[1] - a[1] - shift.y as f32 * 32.).abs() < 0.001);
        }
    }

    #[test]
    fn atlas_paint_other_domains_and_distant_repeated_art_are_preserved() {
        let g = GridGeometry {
            width: 12,
            height: 12,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut mesh = fixture(&g, [0, 0], 40000.);
        mesh.uvs[0] = [0.25, 0.5];
        mesh.uvs[1] = crate::scenery_finish::SOURCE_STONE_UV;
        mesh.positions[2][0] += g.tile_width * 32.;
        let original = mesh.clone();
        finish(&mut mesh, &g, [0, 0], &[]);
        assert_eq!(mesh, original);
    }
}
