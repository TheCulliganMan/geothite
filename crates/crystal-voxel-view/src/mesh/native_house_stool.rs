//! The house stool's cushion is the oval at source rows 5..11. Rows above
//! it are background; projecting them over a rectangular cap creates a hole.
use super::*;

pub(super) const PAINT: [[usize; 2]; 4] = [[7, 7], [4, 7], [6, 9], [2, 12]];

pub(super) fn append(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    p: &Placement,
    [w, e, n, s]: [f32; 4],
) {
    let scale = g.tile_height / 8.;
    let center = Vec3::new((w + e) * 0.5, 0., (n + s) * 0.5);
    let radius = ((e - w) * 0.5).min((s - n) * 0.5 - scale);
    let paint = native_source_swatches(p, g, PAINT);
    // All four leg tops lie inside the underside of the cushion, including
    // their outer corners. Their tops overlap it rather than floating below.
    for x in [-0.48, 0.48] {
        for z in [-0.48, 0.48] {
            let first = out.positions.len();
            architecture_box(
                out,
                [
                    center.x + x * radius - 0.4 * scale,
                    center.x + x * radius + 0.4 * scale,
                    0.,
                    3.25 * scale,
                    center.z + z * radius - 0.4 * scale,
                    center.z + z * radius + 0.4 * scale,
                ],
                [1.; 3],
            );
            out.uvs[first..].fill(paint[3]);
        }
    }
    let profile = [
        (0., 3.),
        (0.88, 3.),
        (1., 3.3),
        (1., 4.4),
        (0.9, 4.8),
        (0.6, 5.),
        (0., 5.),
    ];
    for (band, pair) in profile.windows(2).enumerate() {
        let pigment = paint[if band < 3 {
            2
        } else if band < 5 {
            1
        } else {
            0
        }];
        for segment in 0..32 {
            let point = |(r, y): (f32, f32), i: usize| {
                let angle = std::f32::consts::TAU * i as f32 / 32.;
                center
                    + Vec3::new(
                        radius * r * angle.cos(),
                        y * scale,
                        radius * r * angle.sin(),
                    )
            };
            let a = point(pair[0], segment);
            let b = point(pair[1], segment);
            let c = point(pair[1], segment + 1);
            let d = point(pair[0], segment + 1);
            for points in [[a, b, c], [a, c, d]] {
                let normal = (points[1] - points[0]).cross(points[2] - points[0]);
                if normal.length_squared() < 1e-12 {
                    continue;
                }
                let normal = normal.normalize();
                let first = out.positions.len() as u32;
                out.positions.extend(points.map(|p| p.to_array()));
                out.normals.extend([normal.to_array(); 3]);
                out.uvs.extend([pigment; 3]);
                out.colors
                    .extend([crate::interior_models::authored_face_color([1.; 4], normal); 3]);
                out.indices.extend([first, first + 1, first + 2]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
    fn external_native_house_stool_paint_comes_from_the_oval_seat_and_dark_feet() {
        let path = std::path::PathBuf::from(
            std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"),
        );
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
        let pack = loaded.pack();
        let png = image::load_from_memory(&pack.runtime_files()["gfx/tilesets/house.png"])
            .unwrap()
            .to_luma8();
        for ([x, y], expected) in PAINT.into_iter().zip([255, 170, 85, 0]) {
            let tile = [[2, 3], [0x12, 0x13]][y / 8][x / 8];
            assert_eq!(
                png.get_pixel(
                    (tile % 16 * 8 + x % 8) as u32,
                    (tile / 16 * 8 + y % 8) as u32
                )[0],
                expected,
                "highlight, cushion, side and feet retain their own source pigment"
            );
        }
        let map = &pack.data().maps["SafariZoneWardensHome"];
        let art = &pack.runtime_files()["data/tilesets/house_metatiles.bin"];
        let (w, h) = (
            map.attributes.width as usize * 4,
            map.attributes.height as usize * 4,
        );
        let tiles = (0..w * h)
            .map(|i| {
                let (x, y) = (i % w, i / w);
                let block = map.blocks[y / 4 * (w / 4) + x / 4];
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    texture: Default::default(),
                    animation_frames: None,
                    priority: false,
                    source: VisualTileSource {
                        tileset_id: "house".into(),
                        metatile_id: block,
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: art[block as usize * 16 + y % 4 * 4 + x % 4] as u16,
                    },
                }
            })
            .collect::<Vec<_>>();
        let cells = tiles.iter().collect::<Vec<_>>();
        let g = GridGeometry {
            width: w,
            height: h,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let placements = resolve("SafariZoneWardensHome", &cells, &g, None);
        let stools = placements
            .iter()
            .filter(|p| p.kind == ModelKind::Stool)
            .collect::<Vec<_>>();
        assert_eq!(
            stools.len(),
            4,
            "all four source stools remain independent drawings"
        );
        for p in stools {
            let mut mesh = TerrainMeshData {
                footing_heights: vec![2.75; w * h],
                ..Default::default()
            };
            let mut claimed = vec![false; w * h];
            assert!(super::super::append(
                &mut mesh,
                "SafariZoneWardensHome",
                &cells,
                &g,
                p,
                &mut claimed,
                [0, 0]
            ));
            assert_eq!(claimed.iter().filter(|&&value| value).count(), 4);
            assert_eq!(mesh.footing_heights, vec![2.75; w * h]);
            let paint = native_source_swatches(p, &g, PAINT);
            assert!(
                mesh.textured
                    .positions
                    .iter()
                    .zip(&mesh.textured.uvs)
                    .filter(|(position, _)| position[1] > 0.)
                    .all(|(_, uv)| paint.contains(uv))
            );
        }
    }

    #[test]
    fn native_house_stool_is_closed_outward_and_samples_only_its_cushion_and_feet() {
        let g = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let p = Placement {
            kind: ModelKind::Stool,
            column: 0,
            row: 0,
            width: 2,
            height: 2,
            ground: 0,
            depth_pixels: 8.,
            height_pixels: 5.,
            base_pixels: 0.,
            footing_pixels: None,
        };
        for scale in [0.5, 1., 2.] {
            let g = GridGeometry {
                tile_width: 8. * scale,
                tile_height: 8. * scale,
                ..g
            };
            let mut mesh = SurfaceMeshData::default();
            append(&mut mesh, &g, &p, [0., 8. * scale, 0., 8. * scale]);
            let paint = native_source_swatches(&p, &g, PAINT);
            let center = Vec3::new(4. * scale, 4. * scale, 4. * scale);
            let mut projected_top = 0.;
            let mut minimum = Vec3::splat(f32::INFINITY);
            let mut maximum = Vec3::splat(f32::NEG_INFINITY);
            for points in mesh.positions.iter().map(|p| Vec3::from_array(*p)) {
                minimum = minimum.min(points);
                maximum = maximum.max(points);
            }
            assert_eq!(minimum.y, 0.);
            assert_eq!(maximum.y, 5. * scale);
            for tri in mesh.indices.chunks_exact(3) {
                let [a, b, c] =
                    [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let normal = Vec3::from_array(mesh.normals[tri[0] as usize]);
                assert!((b - a).cross(c - a).dot(normal) > 0.);
                assert!(
                    tri.iter()
                        .all(|&i| mesh.uvs[i as usize] == mesh.uvs[tri[0] as usize])
                );
                assert!(
                    paint.contains(&mesh.uvs[tri[0] as usize]),
                    "no background/atlas-border pixels on the cushion"
                );
                if mesh.uvs[tri[0] as usize] != paint[3] {
                    assert!(
                        normal.dot((a + b + c) / 3. - center) > 0.,
                        "every cushion face points outward"
                    );
                    if normal.y > 0. {
                        projected_top += (b - a).cross(c - a).y * 0.5;
                    }
                }
            }
            let expected = 32. * 0.5 * (3. * scale).powi(2) * (std::f32::consts::TAU / 32.).sin();
            assert!(
                (projected_top - expected).abs() < 0.001 * scale * scale,
                "the projected upper surface covers the entire disc, including its center"
            );
        }
    }
}
