use super::*;
use crate::mesh::tests::{frame, source_with_tile};

fn fixture() -> (
    VisualWorldFrame,
    GridGeometry,
    BuildingPlacement,
    TerrainImageSamples,
) {
    fixture_for(Entrance::North)
}

fn fixture_for(
    entrance: Entrance,
) -> (
    VisualWorldFrame,
    GridGeometry,
    BuildingPlacement,
    TerrainImageSamples,
) {
    let (column, row) = match entrance {
        Entrance::North => (0, 4),
        Entrance::West => (4, 0),
    };
    let mut sources = Vec::new();
    for y in 0..16 {
        for x in 0..16 {
            let (block, tile) =
                if (column..column + 12).contains(&x) && (row..row + 12).contains(&y) {
                    (
                        BLOCKS[(y - row) / 4][(x - column) / 4],
                        drawing_tile(x - column, y - row),
                    )
                } else if (entrance == Entrance::North && y < 4 && (4..8).contains(&x))
                    || (entrance == Entrance::West && x < 4 && (8..12).contains(&y))
                {
                    (
                        if entrance == Entrance::North {
                            0x0a
                        } else {
                            0x0b
                        },
                        if (entrance == Entrance::North && y % 4 >= 2)
                            || (entrance == Entrance::West && x % 4 >= 2)
                        {
                            0x4f
                        } else if y % 2 == 1 {
                            if x % 2 == 0 { 6 } else { 0 }
                        } else if x % 2 == 0 {
                            0
                        } else {
                            0x16
                        },
                    )
                } else {
                    (
                        1,
                        if y % 2 == 0 {
                            if x % 2 == 0 { 0 } else { 0x16 }
                        } else if x % 2 == 0 {
                            6
                        } else {
                            0
                        },
                    )
                };
            let mut s = source_with_tile(block, (x % 4) as u8, (y % 4) as u8, tile);
            s.tileset_id = "park".into();
            sources.push(s);
        }
    }
    let f = frame(16, 16, sources);
    let g = GridGeometry {
        width: 16,
        height: 16,
        tile_width: 8.,
        tile_height: 8.,
        origin_x: -64.,
        origin_z: -64.,
    };
    let p = BuildingPlacement {
        column,
        row,
        width: 12,
        height: 12,
        roof_rows: 6,
        ground_tile_index: 0,
    };
    let mut images = TerrainImageSamples::default();
    for tile in &f.tiles {
        let colors = match tile.source.tile_index {
            0x21 => [[140, 45, 50, 255], [220, 110, 115, 255]],
            0x41 => [[170, 130, 120, 255], [235, 225, 200, 255]],
            _ => [[190, 110, 90, 255], [240, 210, 185, 255]],
        };
        images.pixels.insert(
            tile.texture.id(),
            TileImageSample::Rgba((0..64).flat_map(|i| colors[i % 2]).collect()),
        );
    }
    (f, g, p, images)
}

fn empty(g: &GridGeometry) -> TerrainMeshData {
    TerrainMeshData {
        authored_cells: vec![None; g.width * g.height],
        footing_heights: vec![0.; g.width * g.height],
        ..Default::default()
    }
}

#[test]
fn park_gate_owns_one_whole_building_and_keeps_the_native_north_threshold() {
    let (f, g, p, images) = fixture();
    let before = f.clone();
    let cells = f.tiles.iter().collect::<Vec<_>>();
    let shapes = vec![CellShape::Flat; cells.len()];
    assert!(placements(&cells, &g).contains(&p));
    assert!(super::super::building_placements("NationalPark", &cells, &g).contains(&p));
    let mut mesh = empty(&g);
    let mut claimed = vec![false; cells.len()];
    assert!(super::super::append_building_with_samples(
        &mut mesh,
        &cells,
        &shapes,
        &g,
        "NationalPark",
        p,
        &mut claimed,
        [0, 0],
        Some(&images)
    ));
    assert_eq!(claimed.iter().filter(|v| **v).count(), 144);
    assert_eq!(mesh.footing_heights, vec![0.; cells.len()]);
    for y in 0..g.height {
        for x in 0..g.width {
            assert_eq!(claimed[y * g.width + x], y >= 4 && x < 12);
        }
    }
    assert_eq!(
        mesh.solid.cutaway_ranges,
        vec![0..mesh.solid.positions.len()]
    );
    assert_eq!(mesh.coverage_fade_ranges, mesh.solid.cutaway_ranges);
    assert_eq!(mesh.rigid_buildings.len(), 1);
    assert_eq!(mesh.rigid_buildings[0].doorway, Vec2::new(-16., -40.));
    assert!(
        mesh.solid
            .uvs
            .iter()
            .all(|uv| *uv == crate::scenery_finish::SOURCE_PAINT_UV)
    );
    let repeated = mesh.clone();
    assert!(!append(
        &mut mesh,
        &cells,
        &shapes,
        &g,
        p,
        &mut claimed,
        Some(&images)
    ));
    assert_eq!(mesh, repeated);
    assert_eq!(f, before);
}

#[test]
fn park_gate_refuses_stale_clipped_mixed_and_custom_source_without_partial_claims() {
    let (f, g, p, images) = fixture();
    let indices = (4..16)
        .flat_map(|y| (0..12).map(move |x| y * 16 + x))
        .chain((0..4).flat_map(|y| (4..8).map(move |x| y * 16 + x)))
        .collect::<Vec<_>>();
    for &i in &indices {
        for field in 0..5 {
            let mut tiles = f.tiles.clone();
            let s = &mut tiles[i].source;
            match field {
                0 => s.tile_index ^= 1,
                1 => s.metatile_id ^= 1,
                2 => s.subtile_column ^= 1,
                3 => s.subtile_row ^= 1,
                _ => s.tileset_id = "johto".into(),
            }
            assert!(
                !matches(&tiles.iter().collect::<Vec<_>>(), &g, p),
                "guard {i}/{field}"
            );
        }
    }
    let cells = f.tiles.iter().collect::<Vec<_>>();
    let shapes = vec![CellShape::Flat; cells.len()];
    for invalid in [
        BuildingPlacement { width: 11, ..p },
        BuildingPlacement { row: 0, ..p },
        BuildingPlacement { column: 1, ..p },
    ] {
        assert!(!matches(&cells, &g, invalid));
    }
    let mut mesh = empty(&g);
    let before = mesh.clone();
    let mut claimed = vec![false; cells.len()];
    claimed[4 * 16 + 2] = true;
    let claim_before = claimed.clone();
    assert!(!append(
        &mut mesh,
        &cells,
        &shapes,
        &g,
        p,
        &mut claimed,
        Some(&images)
    ));
    assert_eq!(mesh, before);
    assert_eq!(claimed, claim_before);
    let mut claimed = vec![false; cells.len()];
    assert!(!append(
        &mut mesh,
        &cells,
        &shapes,
        &g,
        p,
        &mut claimed,
        None
    ));
    assert_eq!(mesh, before);
    assert!(claimed.iter().all(|v| !v));
    let mut missing = images.clone();
    missing
        .pixels
        .remove(&f.tiles[(p.row + 7) * 16 + 5].texture.id());
    assert!(!append(
        &mut mesh,
        &cells,
        &shapes,
        &g,
        p,
        &mut claimed,
        Some(&missing)
    ));
    assert_eq!(mesh, before);
    assert!(claimed.iter().all(|v| !v));
}

fn intersects(mesh: &SurfaceMeshData, a: Vec3, b: Vec3) -> bool {
    let ray = (b - a).as_dvec3();
    let a = a.as_dvec3();
    mesh.indices.chunks_exact(3).any(|ids| {
        let [p, q, r] = [ids[0], ids[1], ids[2]]
            .map(|i| Vec3::from_array(mesh.positions[i as usize]).as_dvec3());
        let (e1, e2) = (q - p, r - p);
        let cross = ray.cross(e2);
        let det = e1.dot(cross);
        if det.abs() < 1e-10 {
            return false;
        }
        let offset = a - p;
        let u = offset.dot(cross) / det;
        let cross = offset.cross(e1);
        let v = ray.dot(cross) / det;
        let t = e2.dot(cross) / det;
        u >= -1e-9 && v >= -1e-9 && u + v <= 1. + 1e-9 && t >= 0. && t <= 1.
    })
}

#[test]
fn park_gate_has_a_clear_north_portal_closed_rear_and_outward_faces() {
    let (f, g, p, images) = fixture();
    let cells = f.tiles.iter().collect::<Vec<_>>();
    let day_paint = paint(&cells, &g, p, &images).unwrap();
    let mut mesh = SurfaceMeshData::default();
    geometry(
        &mut mesh,
        [0., 96., 0., 96.],
        8.,
        day_paint,
        Entrance::North,
    );
    for ids in mesh.indices.chunks_exact(3) {
        let [a, b, c] =
            [ids[0], ids[1], ids[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
        let face = (b - a).cross(c - a);
        let normal = ids
            .iter()
            .map(|&i| Vec3::from_array(mesh.normals[i as usize]))
            .sum::<Vec3>();
        assert!(face.length_squared() > 1e-10 && face.dot(normal) > 0.);
    }
    assert!(
        mesh.positions
            .iter()
            .all(|p| p.iter().all(|v| v.is_finite())
                && p[0] >= 0.
                && p[0] <= 96.
                && p[2] >= 0.
                && p[2] <= 96.
                && p[1] >= 0.)
    );
    assert!(
        mesh.normals
            .iter()
            .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 1e-5)
    );
    assert!(
        !intersects(&mesh, Vec3::new(48., 12., -8.), Vec3::new(48., 12., 8.)),
        "north entrance must remain open"
    );
    assert!(intersects(
        &mesh,
        Vec3::new(24., 12., -8.),
        Vec3::new(24., 12., 8.)
    ));
    assert!(
        intersects(&mesh, Vec3::new(48., 12., 104.), Vec3::new(48., 12., 88.)),
        "the native rear is a wall"
    );
    assert!(intersects(
        &mesh,
        Vec3::new(48., 64., 48.),
        Vec3::new(48., 32., 48.)
    ));
    let mut darker = images.clone();
    for sample in darker.pixels.values_mut() {
        if let TileImageSample::Rgba(rgba) = sample {
            for pixel in rgba.chunks_exact_mut(4) {
                for c in &mut pixel[..3] {
                    *c /= 2;
                }
            }
        }
    }
    let mut night = SurfaceMeshData::default();
    geometry(
        &mut night,
        [0., 96., 0., 96.],
        8.,
        paint(&cells, &g, p, &darker).unwrap(),
        Entrance::North,
    );
    assert_eq!(night.positions, mesh.positions);
    assert_eq!(night.indices, mesh.indices);
    assert_ne!(
        night.colors, mesh.colors,
        "live pigment must not become baked prototype paint"
    );
}

#[test]
fn west_gate_meets_its_native_side_threshold_with_a_closed_rotated_shell() {
    let (f, g, p, images) = fixture_for(Entrance::West);
    let cells = f.tiles.iter().collect::<Vec<_>>();
    assert_eq!(entrance(&cells, &g, p), Some(Entrance::West));
    assert!(super::super::building_placements("NationalPark", &cells, &g).contains(&p));
    let shapes = vec![CellShape::Flat; cells.len()];
    let mut mesh = empty(&g);
    let mut claimed = vec![false; cells.len()];
    assert!(super::super::append_building_with_samples(
        &mut mesh,
        &cells,
        &shapes,
        &g,
        "NationalPark",
        p,
        &mut claimed,
        [0, 0],
        Some(&images)
    ));
    assert_eq!(claimed.iter().filter(|v| **v).count(), 144);
    assert_eq!(mesh.rigid_buildings[0].doorway, Vec2::new(-40., 16.));
    assert_eq!(mesh.footing_heights, vec![0.; cells.len()]);
    for y in 0..16 {
        for x in 0..16 {
            assert_eq!(claimed[y * 16 + x], x >= 4 && y < 12);
        }
    }
    let mut solid = SurfaceMeshData::default();
    let paint = paint(&cells, &g, p, &images).unwrap();
    geometry(&mut solid, [0., 96., 0., 96.], 8., paint, Entrance::West);
    assert!(
        !intersects(&solid, Vec3::new(-8., 12., 80.), Vec3::new(8., 12., 80.)),
        "the west portal must be clear at the actual threshold rows"
    );
    assert!(
        intersects(&solid, Vec3::new(-8., 12., 48.), Vec3::new(8., 12., 48.)),
        "the west wall remains solid outside the offset doorway"
    );
    assert!(
        intersects(&solid, Vec3::new(104., 12., 80.), Vec3::new(88., 12., 80.)),
        "the opposite wall stays closed"
    );
    assert!(intersects(
        &solid,
        Vec3::new(48., 64., 48.),
        Vec3::new(48., 32., 48.)
    ));
    // The rotation also supports non-square source cells. Inverse-transpose
    // normals and original winding must agree after that anisotropic transform.
    let mut stretched = SurfaceMeshData::default();
    geometry(
        &mut stretched,
        [0., 120., 0., 96.],
        8.,
        paint,
        Entrance::West,
    );
    for normal in &stretched.normals {
        assert!((Vec3::from_array(*normal).length() - 1.).abs() < 1e-5);
    }
    for tri in stretched.indices.chunks_exact(3) {
        let [a, b, c] =
            [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(stretched.positions[i as usize]));
        let face_normal = (b - a).cross(c - a);
        assert!(face_normal.length_squared() > 1e-12);
        for &i in tri {
            assert!(face_normal.dot(Vec3::from_array(stretched.normals[i as usize])) > 0.);
        }
    }
    let guard = (0..12)
        .flat_map(|y| (4..16).map(move |x| y * 16 + x))
        .chain((8..12).flat_map(|y| (0..4).map(move |x| y * 16 + x)));
    for i in guard {
        for field in 0..5 {
            let mut tiles = f.tiles.clone();
            let s = &mut tiles[i].source;
            match field {
                0 => s.tile_index ^= 1,
                1 => s.metatile_id ^= 1,
                2 => s.subtile_column ^= 1,
                3 => s.subtile_row ^= 1,
                _ => s.tileset_id = "johto".into(),
            }
            assert!(
                !matches(&tiles.iter().collect::<Vec<_>>(), &g, p),
                "west guard {i}/{field}"
            );
        }
    }
}

#[test]
fn park_gate_roof_is_closed_from_below_and_both_gable_ends() {
    let (f, g, p, images) = fixture();
    let cells = f.tiles.iter().collect::<Vec<_>>();
    let mut mesh = SurfaceMeshData::default();
    roof_shell(
        &mut mesh,
        [-24., 72., -40., 56.],
        32.,
        48.,
        8.,
        paint(&cells, &g, p, &images).unwrap(),
    );
    let mut points = std::collections::HashMap::new();
    let mut edges = std::collections::HashMap::<(usize, usize), (usize, i32)>::new();
    let origin = Vec3::from_array(mesh.positions[0]).as_dvec3();
    let mut volume = 0.;
    for ids in mesh.indices.chunks_exact(3) {
        let p = [ids[0], ids[1], ids[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
        let [a, b, c] = p.map(|p| p.as_dvec3() - origin);
        volume += a.dot(b.cross(c));
        let ids = p.map(|p| {
            let count = points.len();
            *points
                .entry(p.to_array().map(f32::to_bits))
                .or_insert(count)
        });
        for (a, b) in [(ids[0], ids[1]), (ids[1], ids[2]), (ids[2], ids[0])] {
            let key = if a < b { (a, b) } else { (b, a) };
            let edge = edges.entry(key).or_default();
            edge.0 += 1;
            edge.1 += if a < b { 1 } else { -1 };
        }
    }
    assert!(volume > 0.);
    assert!(edges.values().all(|&(n, d)| n == 2 && d == 0));
    for (a, b) in [
        (Vec3::new(24., 24., 8.), Vec3::new(24., 40., 8.)),
        (Vec3::new(-32., 36., 8.), Vec3::new(0., 36., 8.)),
        (Vec3::new(80., 36., 8.), Vec3::new(48., 36., 8.)),
    ] {
        assert!(intersects(&mesh, a, b));
    }
}

#[test]
fn park_threshold_finishes_both_native_orientations_without_changing_floor_geometry() {
    for side in [false, true] {
        let (mut f, mut g, _, mut images) = fixture();
        f.tiles.truncate(16);
        g.width = 4;
        g.height = 4;
        for (i, tile) in f.tiles.iter_mut().enumerate() {
            let (x, y) = (i % 4, i / 4);
            tile.column = x as u32;
            tile.row = y as u32;
            tile.source.metatile_id = if side { 0x0b } else { 0x0a };
            tile.source.subtile_column = x as u8;
            tile.source.subtile_row = y as u8;
            tile.source.tile_index = if if side { x >= 2 } else { y >= 2 } {
                0x4f
            } else if y % 2 == 0 {
                if x % 2 == 0 { 0 } else { 0x16 }
            } else if x % 2 == 0 {
                6
            } else {
                0
            };
            images.pixels.insert(
                tile.texture.id(),
                TileImageSample::Rgba(
                    [[220, 70, 65, 255], [240, 140, 120, 255]]
                        .repeat(32)
                        .into_iter()
                        .flatten()
                        .collect(),
                ),
            );
        }
        let mut original = empty(&g);
        for i in 0..16 {
            let (w, e, n, s) = g.bounds(i % 4, i / 4);
            append_top(&mut original.textured, [w, e, n, s], 0., g.uv(i % 4, i / 4));
        }
        let mut out = original.clone();
        crate::mesh::outdoor_entry_mat::finish(
            &mut out,
            &f.tiles.iter().collect::<Vec<_>>(),
            &g,
            &images,
        );
        assert_eq!(out.footing_heights, original.footing_heights);
        assert_eq!(out.textured.positions, original.textured.positions);
        assert_eq!(
            out.solid.indices.len() + out.textured.indices.len(),
            original.textured.indices.len()
        );
        assert_eq!(out.authored_cells.iter().filter(|v| v.is_some()).count(), 8);
        assert!(
            out.solid
                .uvs
                .iter()
                .all(|uv| crate::scenery_finish::source_fabric_coordinates(*uv).is_some())
        );
        for i in 0..16 {
            let mut changed = f.tiles.clone();
            changed[i].source.tile_index ^= 1;
            let mut refused = original.clone();
            crate::mesh::outdoor_entry_mat::finish(
                &mut refused,
                &changed.iter().collect::<Vec<_>>(),
                &g,
                &images,
            );
            assert_eq!(refused, original, "partial threshold {i}/{side}");
        }
    }
}

#[test]
#[ignore = "requires the completed external CRYSTAL_RENDER_TEST_PACK"]
fn external_park_gate_uses_actual_drawing_and_palette_in_both_map_states() {
    let path = std::env::var("CRYSTAL_RENDER_TEST_PACK").expect("set the completed external pack");
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
    let pack = loaded.pack();
    let layout = &pack.runtime_files()["data/tilesets/park_metatiles.bin"];
    let sheet = image::load_from_memory(&pack.runtime_files()["gfx/tilesets/park.png"])
        .unwrap()
        .to_rgba8();
    for name in ["NationalPark", "NationalParkBugContest"] {
        let module = &pack.data().maps[name];
        for (origin, expected, side) in [
            ([12, 88], [4, 8], Entrance::North),
            ([60, 20], [8, 8], Entrance::West),
        ] {
            let mut sources = Vec::new();
            for y in origin[1]..origin[1] + 20 {
                for x in origin[0]..origin[0] + 20 {
                    let b = module.blocks[y / 4 * module.attributes.width as usize + x / 4];
                    let tile = layout[b as usize * 16 + y % 4 * 4 + x % 4] as u16;
                    let mut source = source_with_tile(b, (x % 4) as u8, (y % 4) as u8, tile);
                    source.tileset_id = "park".into();
                    sources.push(source);
                }
            }
            let mut f = frame(20, 20, sources);
            f.map_id = name.into();
            f.grid_origin = bevy::prelude::IVec2::new(origin[0] as i32, origin[1] as i32);
            let g = GridGeometry {
                width: 20,
                height: 20,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: -80.,
                origin_z: -80.,
            };
            let mut images = TerrainImageSamples::default();
            for tile in &f.tiles {
                let id = tile.source.tile_index as u32;
                let rgba = image::imageops::crop_imm(
                    &sheet,
                    id % (sheet.width() / 8) * 8,
                    id / (sheet.width() / 8) * 8,
                    8,
                    8,
                )
                .to_image()
                .into_raw();
                images
                    .pixels
                    .insert(tile.texture.id(), TileImageSample::Rgba(rgba));
            }
            let cells = f.tiles.iter().collect::<Vec<_>>();
            let found = super::super::building_placements(name, &cells, &g);
            let p = *found
                .iter()
                .find(|&&p| matches(&cells, &g, p))
                .expect("actual gate must reach its production owner");
            assert_eq!(
                (p.column, p.row, p.width, p.height),
                (expected[0], expected[1], 12, 12)
            );
            assert_eq!(entrance(&cells, &g, p), Some(side));
            let shapes = cells
                .iter()
                .map(|t| shape_for_source(&t.source))
                .collect::<Vec<_>>();
            let mut mesh = empty(&g);
            let mut claimed = vec![false; cells.len()];
            assert!(super::super::append_building_with_samples(
                &mut mesh,
                &cells,
                &shapes,
                &g,
                name,
                p,
                &mut claimed,
                [origin[0] as i32, origin[1] as i32],
                Some(&images)
            ));
            assert_eq!(claimed.iter().filter(|v| **v).count(), 144);
            assert_eq!(mesh.rigid_buildings.len(), 1);
            assert!(mesh.solid.positions.iter().any(|p| p[1] > 40.));
            assert_eq!(mesh.footing_heights, vec![0.; cells.len()]);
        }
    }
}
