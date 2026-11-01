//! Source-painted, shallow aggregate instead of magnified square floor pixels.
use super::*;

pub(super) fn source(map: &str, s: &VisualTileSource) -> bool {
    if map != "PowerPlant"
        || s.tileset_id.as_ref() != "facility"
        || s.tile_index != 0x1c
        || s.subtile_column >= 4
        || s.subtile_row >= 4
    {
        return false;
    }
    let mask: u16 = match s.metatile_id {
        0x1a => 0xcc00,
        0x1b => 0xffff,
        0x1c | 0x2e => 0xcccc,
        0x2c => 0xccff,
        _ => 0,
    };
    mask & (1 << (s.subtile_row * 4 + s.subtile_column)) != 0
}

fn fleck(x: usize, z: usize) -> Option<usize> {
    match (x, z) {
        (0, 0) | (4, 3) | (3, 4) | (2, 5) | (7, 7) => Some(0),
        (6, 0) | (7, 1) | (0, 2) | (1, 2) | (5, 4) | (4, 5) | (3, 6) => Some(6),
        _ => None,
    }
}

pub(super) fn append(
    mesh: &mut SurfaceMeshData,
    b: [f32; 4],
    height: f32,
    uv: (f32, f32, f32, f32),
) {
    let [w, e, n, s] = b;
    let depth = (e - w).min(s - n) * 0.003;
    let at = |x: f32, z: f32, h: f32| [w + (e - w) * x / 8., height - h, n + (s - n) * z / 8.];
    // All three paints come from this actual live source tile, including
    // palette changes. Never interpolate through unrelated atlas pixels.
    let paint = |x: usize| {
        [
            uv.0 + (uv.1 - uv.0) * (x as f32 + 0.5) / 8.,
            uv.2 + (uv.3 - uv.2) * 0.5 / 8.,
        ]
    };
    let mut triangle = |p: [[f32; 3]; 3], sample: [f32; 2]| {
        let n = (Vec3::from_array(p[1]) - Vec3::from_array(p[0]))
            .cross(Vec3::from_array(p[2]) - Vec3::from_array(p[0]))
            .normalize()
            .to_array();
        let first = mesh.positions.len() as u32;
        mesh.positions.extend(p);
        mesh.normals.extend([n; 3]);
        mesh.uvs.extend([sample; 3]);
        mesh.colors.extend([TEXTURED_SHADE; 3]);
        mesh.indices.extend([first, first + 1, first + 2]);
    };
    for z in 0..8 {
        let mut x = 0;
        while x < 8 {
            if let Some(ink) = fleck(x, z) {
                let (x, z) = (x as f32, z as f32);
                let outer = [
                    at(x, z, 0.),
                    at(x, z + 1., 0.),
                    at(x + 1., z + 1., 0.),
                    at(x + 1., z, 0.),
                ];
                let inner = [
                    at(x + 0.16, z + 0.5, depth),
                    at(x + 0.5, z + 0.84, depth),
                    at(x + 0.84, z + 0.5, depth),
                    at(x + 0.5, z + 0.16, depth),
                ];
                let center = at(x + 0.5, z + 0.5, depth * 0.4);
                for i in 0..4 {
                    let j = (i + 1) % 4;
                    triangle([outer[i], outer[j], inner[i]], paint(1));
                    triangle([outer[j], inner[j], inner[i]], paint(1));
                    triangle([inner[i], inner[j], center], paint(ink));
                }
            } else {
                // Merge the uninterrupted blue field; no artificial slab
                // seams or extra height are introduced between source cells.
                let first = x;
                while x + 1 < 8 && fleck(x + 1, z).is_none() {
                    x += 1;
                }
                let (a, b, z) = (first as f32, (x + 1) as f32, z as f32);
                triangle(
                    [at(a, z, 0.), at(a, z + 1., 0.), at(b, z + 1., 0.)],
                    paint(1),
                );
                triangle([at(a, z, 0.), at(b, z + 1., 0.), at(b, z, 0.)], paint(1));
            }
            x += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aggregate_floor_masks_cover_only_the_actual_external_source_phases() {
        let pack = crystal_assets::read_loaded_verified_compiled_game_pack(
            std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap(),
        )
        .unwrap();
        let art = &pack.pack().runtime_files()["data/tilesets/facility_metatiles.bin"];
        for (block, tiles) in art.chunks_exact(16).enumerate() {
            for (i, &tile) in tiles.iter().enumerate() {
                let mut s = VisualTileSource {
                    tileset_id: "facility".into(),
                    metatile_id: block as u16,
                    subtile_column: (i % 4) as u8,
                    subtile_row: (i / 4) as u8,
                    tile_index: tile as u16,
                };
                assert_eq!(
                    source("PowerPlant", &s),
                    tile == 0x1c,
                    "block{block:x} phase{i}"
                );
                for map in [
                    "SilphCo1F",
                    "TrainerHouseB1F",
                    "TeamRocketBaseB2F",
                    "MrPokemonsHouse",
                ] {
                    assert!(!source(map, &s));
                }
                s.tile_index = 0x91;
                assert!(!source("PowerPlant", &s));
                s.tile_index = 0x1c;
                s.subtile_row = 4;
                assert!(!source("PowerPlant", &s));
            }
        }
    }
    #[test]
    fn aggregate_floor_has_a_complete_shallow_surface_and_source_pigments_at_both_scales() {
        for size in [8., 32.] {
            let mut mesh = SurfaceMeshData::default();
            append(&mut mesh, [0., size, 0., size], 3.25, (0., 1., 0., 1.));
            let mut areas = [0.; 3];
            for t in mesh.indices.chunks_exact(3) {
                let p = [t[0], t[1], t[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let cross = (p[1] - p[0]).cross(p[2] - p[0]);
                assert!(cross.y > 0.);
                let normal = Vec3::from_array(mesh.normals[t[0] as usize]);
                assert!((normal - cross.normalize()).length() < 0.00001);
                let uv = mesh.uvs[t[0] as usize];
                assert!(t.iter().all(|&i| mesh.uvs[i as usize] == uv));
                let ink = if uv == [0.0625, 0.0625] {
                    0
                } else if uv == [0.1875, 0.0625] {
                    1
                } else {
                    assert_eq!(uv, [0.8125, 0.0625]);
                    2
                };
                areas[ink] += cross.y * 0.5;
                if ink != 1 {
                    let center = (p[0] + p[1] + p[2]) / 3.;
                    let x = (center.x / size * 8.).floor() as usize;
                    let z = (center.z / size * 8.).floor() as usize;
                    assert_eq!(
                        fleck(x, z),
                        Some(if ink == 0 { 0 } else { 6 }),
                        "preserve the actual light/dark aggregate centers"
                    );
                }
            }
            let area = size * size;
            assert!(
                (areas.iter().sum::<f32>() - area).abs() < area * 0.00001,
                "no holes or overlapping floor layers"
            );
            assert!((areas[0] / area - 5. * 0.68 * 0.68 / 2. / 64.).abs() < 0.00001);
            assert!((areas[2] / area - 7. * 0.68 * 0.68 / 2. / 64.).abs() < 0.00001);
            assert!(mesh.positions.iter().all(|p| p[0] >= 0.
                && p[0] <= size
                && p[2] >= 0.
                && p[2] <= size
                && p[1] <= 3.25
                && p[1] >= 3.25 - size * 0.003 - 0.00001));
        }
    }
    #[test]
    fn aggregate_floor_finishing_preserves_support_claims_and_custom_ownership() {
        let g = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let tiles = (0..16)
            .map(|i| VisualTile {
                column: (i % 4) as u32,
                row: (i / 4) as u32,
                texture: Default::default(),
                priority: false,
                animation_frames: None,
                source: VisualTileSource {
                    tileset_id: "facility".into(),
                    metatile_id: 0x1b,
                    subtile_column: (i % 4) as u8,
                    subtile_row: (i / 4) as u8,
                    tile_index: 0x1c,
                },
            })
            .collect::<Vec<_>>();
        let mut original = TerrainMeshData {
            footing_heights: vec![3.25; 16],
            authored_cells: vec![None; 16],
            ..Default::default()
        };
        for i in 0..16 {
            append_top(
                &mut original.textured,
                g.bounds(i % 4, i / 4).into(),
                3.25,
                g.uv(i % 4, i / 4),
            );
        }
        let refs = tiles.iter().collect::<Vec<_>>();
        let source_before = tiles.clone();
        let mut mesh = original.clone();
        assert_eq!(
            finish_surfaces(&mut mesh, "PowerPlant", &refs, &g, [-3, 7]),
            16
        );
        assert_eq!(mesh.footing_heights, original.footing_heights);
        assert_eq!(mesh.authored_cells, original.authored_cells);
        assert_eq!(tiles, source_before);
        let custom:Document=serde_json::from_str(r#"{"objects":[{"name":"Custom aggregate","map":"PowerPlant","tileset":"facility","metatile":27,"origin":[0,0],"tiles":[[28]],"ground":28,"top_pixels":0,"depth_pixels":0}]}"#).unwrap();
        let mut owned = original.clone();
        assert_eq!(
            finish_surfaces_with_profiles(
                &mut owned,
                "PowerPlant",
                &refs,
                &g,
                [0, 0],
                Some(&custom)
            ),
            15
        );
        assert_eq!(&owned.textured.indices[..6], &[0, 1, 2, 0, 2, 3]);
        assert_eq!(
            &owned.textured.positions[..4],
            &original.textured.positions[..4]
        );
        assert_eq!(&owned.textured.uvs[..4], &original.textured.uvs[..4]);
        assert_eq!(owned.footing_heights, original.footing_heights);
        assert_eq!(owned.authored_cells, original.authored_cells);
        let mut other = original.clone();
        assert_eq!(
            finish_surfaces(&mut other, "MrPokemonsHouse", &refs, &g, [0, 0]),
            0
        );
        assert_eq!(other, original);
    }
}
