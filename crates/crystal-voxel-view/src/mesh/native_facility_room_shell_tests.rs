use super::*;

fn fixture(
    map: &str,
    pack: &crystal_assets::LoadedCompiledGamePack,
) -> (Vec<VisualTile>, GridGeometry, VisualWorldFrame) {
    let pack = pack.pack();
    let data = &pack.data().maps[map];
    let art = &pack.runtime_files()["data/tilesets/facility_metatiles.bin"];
    let (w, h) = (
        data.attributes.width as usize * 4,
        data.attributes.height as usize * 4,
    );
    let tiles = (0..w * h)
        .map(|i| {
            let (x, y) = (i % w, i / w);
            let b = data.blocks[y / 4 * (w / 4) + x / 4];
            VisualTile {
                column: x as u32,
                row: y as u32,
                texture: Default::default(),
                priority: false,
                animation_frames: None,
                source: VisualTileSource {
                    tileset_id: "facility".into(),
                    metatile_id: b,
                    subtile_column: (x % 4) as u8,
                    subtile_row: (y % 4) as u8,
                    tile_index: art[b as usize * 16 + y % 4 * 4 + x % 4] as u16,
                },
            }
        })
        .collect::<Vec<_>>();
    let g = GridGeometry {
        width: w,
        height: h,
        tile_width: 8.,
        tile_height: 8.,
        origin_x: 0.,
        origin_z: 0.,
    };
    let mut f = crate::mesh::tests::frame(
        w as u32,
        h as u32,
        tiles.iter().map(|t| t.source.clone()).collect(),
    );
    f.map_id = map.into();
    f.source_map_size_core_tiles = Some(UVec2::new(w as u32 / 2, h as u32 / 2));
    f.boundary_warps = data
        .events
        .warps
        .iter()
        .map(|warp| IVec2::new(warp.x.into(), warp.y.into()))
        .collect::<Vec<_>>()
        .into();
    (tiles, g, f)
}

fn ray(surface: &SurfaceMeshData, axis: usize, point: [f32; 2], depth: [f32; 2]) -> bool {
    let along = if axis == 2 { 0 } else { 2 };
    surface.indices.chunks_exact(3).any(|t| {
        let [a, b, c] = [0, 1, 2].map(|j| surface.positions[t[j] as usize]);
        let u = [b[along] - a[along], b[1] - a[1]];
        let v = [c[along] - a[along], c[1] - a[1]];
        let p = [point[0] - a[along], point[1] - a[1]];
        let det = u[0] * v[1] - u[1] * v[0];
        if det.abs() < 0.000001 {
            return false;
        }
        let s = (p[0] * v[1] - p[1] * v[0]) / det;
        let q = (u[0] * p[1] - u[1] * p[0]) / det;
        let d = a[axis] + s * (b[axis] - a[axis]) + q * (c[axis] - a[axis]);
        s >= -0.0001 && q >= -0.0001 && s + q <= 1.0001 && d >= depth[0] && d <= depth[1]
    })
}

#[test]
fn native_facility_room_shell_closes_rear_cabinets_side_joins_and_retains_exit_and_support() {
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap(),
    )
    .unwrap();
    for map in ["MrPokemonsHouse", "RuinsOfAlphResearchCenter"] {
        let (tiles, g, f) = fixture(map, &loaded);
        let before = tiles.clone();
        let refs = tiles.iter().collect::<Vec<_>>();
        let mut mesh = TerrainMeshData {
            footing_heights: vec![2.75; tiles.len()],
            authored_cells: vec![None; tiles.len()],
            ..Default::default()
        };
        assert!(append(&mut mesh, &f, &refs, &g, None));
        assert_eq!(tiles, before);
        assert_eq!(mesh.footing_heights, vec![2.75; tiles.len()]);
        assert_eq!(mesh.authored_cells, vec![None; tiles.len()]);
        assert!(mesh.solid.indices.is_empty());
        assert_eq!(
            mesh.textured
                .positions
                .iter()
                .map(|p| p[1])
                .fold(0., f32::max),
            32.
        );
        for x in 0..16 {
            for dx in [0.1, 0.5, 0.9] {
                for y in [1., 7., 15., 23., 31.] {
                    assert!(
                        ray(&mesh.textured, 2, [(x as f32 + dx) * 8., y], [12.9, 16.2]),
                        "{map}: full rear backing behind every source column, including furniture"
                    );
                }
            }
        }
        for z in 2..16 {
            for y in [1., 15., 31.] {
                assert!(
                    ray(&mesh.textured, 0, [(z as f32 + 0.5) * 8., y], [-3.1, 0.2]),
                    "{map}: west join"
                );
                assert!(
                    ray(
                        &mesh.textured,
                        0,
                        [(z as f32 + 0.5) * 8., y],
                        [127.8, 131.1]
                    ),
                    "{map}: east join"
                );
            }
        }
        for x in [40., 56.] {
            assert!(
                !ray(&mesh.textured, 2, [x, 15.], [127.8, 131.1]),
                "both authoritative exit tiles remain open"
            );
        }
        for x in [8., 24., 72., 120.] {
            assert!(ray(&mesh.textured, 2, [x, 15.], [127.8, 131.1]));
        }
        for (range, _) in &mesh.camera_wall_textured_ranges {
            assert!(mesh.textured.cutaway_ranges.contains(range));
        }
        for (range, plane) in &mesh.camera_wall_textured_ranges {
            for p in &mesh.textured.positions[range.clone()] {
                assert!(
                    plane[0] * p[0] + plane[1] * p[2] <= plane[2] + 0.001,
                    "wall body and recessed trim stay behind the physical boundary"
                );
            }
        }
        assert_eq!(
            mesh.camera_wall_textured_ranges.len(),
            mesh.textured.cutaway_ranges.len()
        );
        for t in mesh.textured.indices.chunks_exact(3) {
            let [a, b, c] =
                [0, 1, 2].map(|j| Vec3::from_array(mesh.textured.positions[t[j] as usize]));
            let n = Vec3::from_array(mesh.textured.normals[t[0] as usize]);
            assert!((b - a).cross(c - a).dot(n) > 0.);
        }
        // Only the green wall's own field, light trim and ink donors are used.
        let donor = source(&f, &refs, &g, None).unwrap();
        let uv = g.uv(donor % g.width, donor / g.width);
        for uv2 in &mesh.textured.uvs {
            assert!((uv2[0] - (uv.0 + (uv.1 - uv.0) * 3.5 / 8.)).abs() < 0.00001);
            assert!([1.5, 3.5, 4.5]
                .iter()
                .any(|y| (uv2[1] - (uv.2 + (uv.3 - uv.2) * y / 8.)).abs() < 0.00001));
        }
        // Padding changes grid coordinates, never the physical room boundary.
        let pg = GridGeometry {
            width: 24,
            height: 24,
            origin_x: -32.,
            origin_z: -32.,
            ..g
        };
        let mut padded = vec![refs[0]; 24 * 24];
        for y in 0..16 {
            for x in 0..16 {
                padded[(y + 4) * 24 + x + 4] = refs[y * 16 + x];
            }
        }
        let mut pf = f.clone();
        pf.grid_origin = IVec2::new(-4, -4);
        let mut shifted = TerrainMeshData::default();
        assert!(append(&mut shifted, &pf, &padded, &pg, None));
        assert_eq!(shifted.textured.positions, mesh.textured.positions);
        assert_eq!(
            shifted.camera_wall_textured_ranges,
            mesh.camera_wall_textured_ranges
        );
    }
}

#[test]
fn native_facility_room_shell_rejects_edited_clipped_unreviewed_and_custom_north_drawings() {
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap(),
    )
    .unwrap();
    for map in ["MrPokemonsHouse", "RuinsOfAlphResearchCenter"] {
        let (tiles, g, f) = fixture(map, &loaded);
        let refs = tiles.iter().collect::<Vec<_>>();
        assert!(source(&f, &refs, &g, None).is_some());
        for i in 0..64 {
            let mut changed = tiles.clone();
            changed[i].source.tile_index ^= 1;
            assert!(
                source(&f, &changed.iter().collect::<Vec<_>>(), &g, None).is_none(),
                "{map}: entire north drawing is authenticated"
            );
        }
        let mut wrong = f.clone();
        wrong.map_id = "PowerPlant".into();
        assert!(source(&wrong, &refs, &g, None).is_none());
        wrong = f.clone();
        wrong.boundary_warps = Default::default();
        assert!(source(&wrong, &refs, &g, None).is_none());
        assert!(source(&f, &refs[..48], &GridGeometry { height: 3, ..g }, None).is_none());
        let mut custom = built_in_profiles().objects[0].clone();
        custom.name = "Custom rear cabinet".into();
        custom.map = Some(map.into());
        custom.maps = None;
        custom.tileset = "facility".into();
        custom.metatile = tiles[0].source.metatile_id;
        custom.metatiles = None;
        custom.origin = [0, 0];
        custom.tiles = vec![vec![tiles[0].source.tile_index]];
        custom.ground = 1;
        custom.parts.clear();
        custom.footing_pixels = None;
        let profile = Document {
            objects: vec![custom],
            atmosphere: None,
        };
        let mut mesh = TerrainMeshData::default();
        assert!(!append(&mut mesh, &f, &refs, &g, Some(&profile)));
        assert!(mesh.textured.positions.is_empty());
    }
}

#[test]
fn native_facility_room_shell_removes_only_original_flat_wall_art_and_keeps_floor_and_borrowed_donors(
) {
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap(),
    )
    .unwrap();
    for (map, count) in [("MrPokemonsHouse", 16), ("RuinsOfAlphResearchCenter", 14)] {
        let (tiles, g, f) = fixture(map, &loaded);
        let refs = tiles.iter().collect::<Vec<_>>();
        let mut mesh = TerrainMeshData {
            footing_heights: vec![3.25; tiles.len()],
            authored_cells: vec![None; tiles.len()],
            ..Default::default()
        };
        for i in 0..tiles.len() {
            append_top(
                &mut mesh.textured,
                g.bounds(i % g.width, i / g.width).into(),
                0.,
                g.uv(i % g.width, i / g.width),
            );
        }
        let original_vertices = mesh.textured.positions.len();
        assert!(append(&mut mesh, &f, &refs, &g, None));
        let old_triangles = mesh
            .textured
            .indices
            .chunks_exact(3)
            .filter(|t| t.iter().all(|&i| (i as usize) < original_vertices))
            .collect::<Vec<_>>();
        assert_eq!(old_triangles.len(), (tiles.len() - count) * 2);
        for i in 0..tiles.len() {
            let kept = old_triangles
                .iter()
                .filter(|t| {
                    t.iter()
                        .all(|&v| (v as usize) >= i * 4 && (v as usize) < i * 4 + 4)
                })
                .count();
            let wall = i / g.width < 2 && tiles[i].source.tile_index == 0x22;
            assert_eq!(
                kept,
                if wall { 0 } else { 2 },
                "{map}: only native wall paint leaves the floor"
            );
        }
        assert_eq!(mesh.footing_heights, vec![3.25; tiles.len()]);
        assert_eq!(mesh.authored_cells, vec![None; tiles.len()]);
        let donor = source(&f, &refs, &g, None).unwrap();
        let mut borrowed = SurfaceMeshData::default();
        append_top(
            &mut borrowed,
            [40., 48., 80., 88.],
            0.,
            g.uv(donor % g.width, donor / g.width),
        );
        append_top(
            &mut borrowed,
            g.bounds(donor % g.width, donor / g.width).into(),
            2.,
            g.uv(donor % g.width, donor / g.width),
        );
        let before = borrowed.clone();
        remove_flat_wall_drawing(&mut borrowed, &f, &refs, &g);
        assert_eq!(
            borrowed.indices, before.indices,
            "borrowed paint and raised geometry are not wall-ground remnants"
        );
        assert_eq!(borrowed.positions, before.positions);
    }
}

#[test]
fn native_facility_room_shell_power_plant_backing_is_continuous_and_source_guarded() {
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap(),
    ).unwrap();
    let (tiles, g, f) = fixture("PowerPlant", &loaded);
    let refs = tiles.iter().collect::<Vec<_>>();
    assert_eq!((g.width, g.height), (40, 36));
    let mut mesh = TerrainMeshData {
        footing_heights: vec![3.25; tiles.len()],
        authored_cells: vec![None; tiles.len()],
        ..Default::default()
    };
    let source_before = tiles.clone();
    assert!(append(&mut mesh, &f, &refs, &g, None));
    assert_eq!(tiles, source_before);
    assert_eq!(mesh.footing_heights, vec![3.25; tiles.len()]);
    assert_eq!(mesh.authored_cells, vec![None; tiles.len()]);
    for x in 0..40 {
        for dx in [0.1, 0.5, 0.9] {
            for y in [1., 7., 15., 23., 31.] {
                assert!(ray(&mesh.textured, 2, [(x as f32 + dx) * 8., y], [12.9, 16.2]),
                    "continuous backing behind cupboards, sofas, divider join and machines");
            }
        }
    }
    for z in 2..36 {
        for y in [1., 15., 31.] {
            assert!(ray(&mesh.textured, 0, [(z as f32 + 0.5) * 8., y], [-3.1, 0.2]));
            assert!(ray(&mesh.textured, 0, [(z as f32 + 0.5) * 8., y], [319.8, 323.1]));
        }
    }
    for x in [40., 56.] {
        assert!(!ray(&mesh.textured, 2, [x, 15.], [287.8, 291.1]), "both exit tiles stay open");
    }
    for x in [8., 24., 72., 312.] {
        assert!(ray(&mesh.textured, 2, [x, 15.], [287.8, 291.1]));
    }
    for (range, plane) in &mesh.camera_wall_textured_ranges {
        assert!(mesh.textured.cutaway_ranges.contains(range));
        for p in &mesh.textured.positions[range.clone()] {
            assert!(plane[0]*p[0]+plane[1]*p[2] <= plane[2]+0.001);
        }
    }
    let donor = source(&f, &refs, &g, None).unwrap();
    assert_eq!(tiles[donor].source.tile_index, 0x22);
    let uv = g.uv(donor % g.width, donor / g.width);
    assert!(mesh.textured.uvs.iter().all(|p| p[0] > uv.0 && p[0] < uv.1 && p[1] > uv.2 && p[1] < uv.3));
    for i in 0..160 {
        let mut changed = tiles.clone();
        changed[i].source.tile_index ^= 1;
        let mut rejected = TerrainMeshData::default();
        assert!(!append(&mut rejected, &f, &changed.iter().collect::<Vec<_>>(), &g, None));
        assert!(rejected.textured.positions.is_empty());
    }
    let mut wrong = f.clone();
    wrong.boundary_warps = Default::default();
    assert!(source(&wrong, &refs, &g, None).is_none());
    assert!(source(&f, &refs[..120], &GridGeometry { height: 3, ..g }, None).is_none());
    // Only this wall drawing is folded away; the floor and source support survive.
    let mut flat = SurfaceMeshData::default();
    for i in 0..tiles.len() {
        append_top(&mut flat, g.bounds(i%g.width, i/g.width).into(), 0., g.uv(i%g.width, i/g.width));
    }
    remove_flat_wall_drawing(&mut flat, &f, &refs, &g);
    let removed = tiles.iter().enumerate().filter(|(i,t)| i/g.width < 2 && t.source.tile_index==0x22).count();
    assert_eq!(removed, 38);
    assert_eq!(flat.indices.len()/3, (tiles.len()-removed)*2);
}

#[test]
fn native_facility_room_shell_silph_backing_keeps_north_stair_and_real_exits_open() {
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap(),
    ).unwrap();
    let (tiles,g,f)=fixture("SilphCo1F",&loaded);
    assert_eq!((g.width,g.height),(32,16));
    let refs=tiles.iter().collect::<Vec<_>>();
    let mut mesh=TerrainMeshData { footing_heights:vec![2.75;tiles.len()],authored_cells:vec![None;tiles.len()],..Default::default() };
    let before=tiles.clone();
    assert!(append(&mut mesh,&f,&refs,&g,None));
    assert_eq!(tiles,before);
    assert_eq!(mesh.footing_heights,vec![2.75;tiles.len()]);
    assert_eq!(mesh.authored_cells,vec![None;tiles.len()]);
    for x in 0..32 {
        for dx in [0.1,0.5,0.9] {
            for y in [1.,7.,15.,23.,31.] {
                assert_eq!(ray(&mesh.textured,2,[(x as f32+dx)*8.,y],[12.9,16.2]),
                    !matches!(x,26|27),"rear backing continues through furniture, with only the actual stair landing open");
            }
        }
    }
    for z in 2..16 {
        for y in [1.,15.,31.] {
            assert!(ray(&mesh.textured,0,[(z as f32+0.5)*8.,y],[-3.1,0.2]));
            assert!(ray(&mesh.textured,0,[(z as f32+0.5)*8.,y],[255.8,259.1]));
        }
    }
    for x in [40.,56.] {assert!(!ray(&mesh.textured,2,[x,15.],[127.8,131.1]));}
    for x in [8.,24.,72.,248.] {assert!(ray(&mesh.textured,2,[x,15.],[127.8,131.1]));}
    for i in 0..128 {
        let mut changed=tiles.clone();changed[i].source.tile_index^=1;
        let mut rejected=TerrainMeshData::default();
        assert!(!append(&mut rejected,&f,&changed.iter().collect::<Vec<_>>(),&g,None));
        assert!(rejected.textured.positions.is_empty());
    }
    let mut original=SurfaceMeshData::default();
    for i in 0..tiles.len() {
        append_top(&mut original,g.bounds(i%g.width,i/g.width).into(),0.,g.uv(i%g.width,i/g.width));
    }
    remove_flat_wall_drawing(&mut original,&f,&refs,&g);
    assert_eq!(original.indices.len()/3,(tiles.len()-58)*2);
    for y in 0..2 {for x in 26..28 {
        let i=y*g.width+x;
        assert_eq!(original.indices.iter().filter(|&&v| v as usize>=i*4 && (v as usize)<i*4+4).count(),6,
            "the source stair drawing remains intact");
    }}
    let mut custom=built_in_profiles().objects[0].clone();
    custom.name="Custom Silph rear wall".into();custom.map=Some("SilphCo1F".into());custom.maps=None;
    custom.tileset="facility".into();custom.metatile=tiles[0].source.metatile_id;custom.metatiles=None;
    custom.origin=[0,0];custom.tiles=vec![vec![0x22]];custom.ground=1;custom.parts.clear();custom.footing_pixels=None;
    let mut rejected=TerrainMeshData::default();
    assert!(!append(&mut rejected,&f,&refs,&g,Some(&Document {objects:vec![custom],atmosphere:None})));
    assert!(rejected.textured.positions.is_empty());
}

#[test]
fn native_facility_room_shell_trainer_basement_backs_cabinets_and_joins_arena_without_sealing_stairs() {
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap(),
    ).unwrap();
    let (tiles, base, f)=fixture("TrainerHouseB1F",&loaded);
    assert_eq!((base.width,base.height),(20,32));
    let refs=tiles.iter().collect::<Vec<_>>();
    assert_eq!(tiles[source(&f,&refs,&base,None).unwrap()].source.tile_index,0x22);
    for tile_size in [8.,32.] {
        let g=GridGeometry {tile_width:tile_size,tile_height:tile_size,..base};
        let mut mesh=TerrainMeshData {footing_heights:vec![3.25;tiles.len()],authored_cells:vec![None;tiles.len()],..Default::default()};
        let before=tiles.clone();
        assert!(append(&mut mesh,&f,&refs,&g,None));
        assert_eq!(tiles,before);
        assert_eq!(mesh.footing_heights,vec![3.25;tiles.len()]);
        assert_eq!(mesh.authored_cells,vec![None;tiles.len()]);
        for x in 0..20 {for dx in [0.1,0.5,0.9] {for source_y in [1.,7.,15.,23.,31.] {
            assert!(ray(&mesh.textured,2,[(x as f32+dx)*tile_size,source_y*tile_size/8.],[1.6*tile_size,2.02*tile_size]),
                "rear backing continues behind all four cabinets at source and production scale");
        }}}
        for z in 2..12 {for source_y in [1.,15.,31.] {
            assert!(ray(&mesh.textured,0,[(z as f32+0.5)*tile_size,source_y*tile_size/8.],[-0.4*tile_size,0.02*tile_size]));
            assert_eq!(ray(&mesh.textured,0,[(z as f32+0.5)*tile_size,source_y*tile_size/8.],[19.98*tile_size,20.4*tile_size]),
                !matches!(z,8|9),"the actual east stair landing stays open");
        }}
        assert!(mesh.textured.positions.iter().all(|p|p[2]<=12.*tile_size+0.001),
            "the authored lower arena owns its enclosure without a second green wall");
        for (range,plane) in &mesh.camera_wall_textured_ranges {
            assert!(mesh.textured.cutaway_ranges.contains(range));
            for p in &mesh.textured.positions[range.clone()] {
                assert!(plane[0]*p[0]+plane[1]*p[2]<=plane[2]+0.001);
            }
        }
    }
    let guarded=(0..4).flat_map(|y|(0..20).map(move |x|y*20+x))
        .chain([(16,8),(0,12),(16,12)].into_iter().flat_map(|(x,y)|
            (0..4).flat_map(move |dy|(0..4).map(move |dx|(y+dy)*20+x+dx)))).collect::<Vec<_>>();
    assert_eq!(guarded.len(),128);
    for i in guarded {
        let mut changed=tiles.clone();changed[i].source.tile_index^=1;
        let mut rejected=TerrainMeshData::default();
        assert!(!append(&mut rejected,&f,&changed.iter().collect::<Vec<_>>(),&base,None));
        assert!(rejected.textured.positions.is_empty());
    }
    let mut wrong=f.clone();wrong.boundary_warps=Default::default();
    assert!(source(&wrong,&refs,&base,None).is_none());
    let mut flat=SurfaceMeshData::default();
    for i in 0..tiles.len() {append_top(&mut flat,base.bounds(i%20,i/20).into(),0.,base.uv(i%20,i/20));}
    remove_flat_wall_drawing(&mut flat,&f,&refs,&base);
    assert_eq!(flat.indices.len()/3,(tiles.len()-22)*2);
    for y in 8..10 {for x in 18..20 {
        let i=y*20+x;
        assert_eq!(flat.indices.iter().filter(|&&v|v as usize>=i*4&&(v as usize)<i*4+4).count(),6,
            "stair source art remains intact");
    }}
}
