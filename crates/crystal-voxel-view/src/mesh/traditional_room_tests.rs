use super::*;
use std::sync::Arc;
fn tile(x: usize, y: usize, block: u16, art: u16) -> VisualTile {
    VisualTile {
        animation_frames: None,
        column: x as u32,
        row: y as u32,
        source: VisualTileSource {
            tileset_id: Arc::from("traditional_house"),
            metatile_id: block,
            subtile_column: (x % 4) as u8,
            subtile_row: (y % 4) as u8,
            tile_index: art,
        },
        texture: Default::default(),
        priority: false,
    }
}
fn floor_fixture() -> (Vec<VisualTile>, GridGeometry) {
    let art = [
        [0x44, 0x45, 0x45, 0x46],
        [0x54, 0x55, 0x55, 0x56],
        [0x45, 0x46, 0x44, 0x45],
        [0x55, 0x56, 0x54, 0x55],
    ];
    (
        (0..16)
            .map(|i| tile(i % 4, i / 4, 4, art[i / 4][i % 4]))
            .collect(),
        GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -7.,
            origin_z: 3.,
        },
    )
}
#[test]
fn mixed_tatami_drawings_preserve_floor_around_real_props_and_reject_damage() {
    for map in source::TATAMI_ROOMS {
        for (block, count) in [
            (0x0c, 8),
            (0x14, 8),
            (0x23, 4),
            (0x33, 4),
            (0x35, 4),
            (0x36, 10),
            (0x0e, 12),
            (0x10, 12),
            (0x11, 10),
            (0x15, 10),
            (0x18, 10),
            (0x19, 6),
            (0x1b, 8),
            (0x1c, 12),
            (0x2e, 8),
            (0x2f, 8),
            (0x30, 8),
        ] {
            let (mut tiles, g) = floor_fixture();
            let drawing = source::tatami_drawing(block).unwrap();
            for (i, t) in tiles.iter_mut().enumerate() {
                t.source.metatile_id = block;
                t.source.tile_index = drawing[i / 4][i % 4];
            }
            let cells: Vec<_> = tiles.iter().collect();
            let mask = tatami_floor_mask(map, &cells, &g, [0, 0], &[]);
            assert_eq!(mask.iter().filter(|&&v| v).count(), count, "{map} block {block:#x}");
            let props: Vec<_> = mask.iter().map(|v| !*v).collect();
            assert_eq!(
                tatami_floor_mask(map, &cells, &g, [0, 0], &props),
                mask,
                "real prop ownership must not recolor the surrounding floor"
            );
            for i in 0..16 {
                let mut changed = tiles.clone();
                changed[i].source.tile_index ^= 1;
                assert!(tatami_floor_mask(
                    map,
                    &changed.iter().collect::<Vec<_>>(),
                    &g,
                    [0, 0],
                    &[]
                )
                .iter()
                .all(|v| !*v));
                if mask[i] {
                    let mut excluded = props.clone();
                    excluded[i] = true;
                    assert!(tatami_floor_mask(map, &cells, &g, [0, 0], &excluded)
                        .iter()
                        .all(|v| !*v));
                }
            }
        }
    }
}
#[test]
fn continuous_tatami_finishes_all_sixteen_cells_without_height_or_object_credit() {
    for map in source::TATAMI_ROOMS {
        let (t, g) = floor_fixture();
        let c: Vec<_> = t.iter().collect();
        let mut mesh = TerrainMeshData::default();
        mesh.authored_cells = vec![None; 16];
        mesh.footing_heights = vec![3.25; 16];
        for y in 0..4 {
            for x in 0..4 {
                append_top(&mut mesh.textured, g.bounds(x, y).into(), 0., g.uv(x, y));
            }
        }
        let original_floor = mesh.textured.clone();
        assert_eq!(
            modeled_interiors::finish_surfaces(&mut mesh, map, &c, &g, [0, 0]),
            16
        );
        assert_eq!(mesh.textured.indices.len(), 16 * 6);
        assert_eq!(mesh.textured.positions, original_floor.positions);
        assert_eq!(mesh.textured.uvs, original_floor.uvs);
        assert_eq!(mesh.textured.normals, original_floor.normals);
        assert_eq!(mesh.textured.indices, original_floor.indices);
        assert!(mesh.textured.positions.iter().all(|p| p[1] == 0.));
        for y in 0..4 {
            for x in 0..4 {
                let (u0, u1, v0, v1) = g.uv(x, y);
                for uv in [[u0, v0], [u0, v1], [u1, v0], [u1, v1]] {
                    assert!(mesh.textured.uvs.contains(&uv));
                }
            }
        }
        assert!(mesh.solid.positions.iter().all(|p| p[1] == 0.));
        assert_eq!(mesh.authored_cells, vec![None; 16]);
        assert_eq!(mesh.footing_heights, vec![3.25; 16]);
        assert!(traditional_house_cushion_placements(map, &c, &g).is_empty());
    }
}
#[test]
fn tatami_rejects_changed_drawing_phase_custom_height_and_distorted_uv() {
    for map in source::TATAMI_ROOMS {
        let (t, g) = floor_fixture();
        let c: Vec<_> = t.iter().collect();
        assert_eq!(
            tatami_floor_mask(map, &c, &g, [0, 0], &[])
                .iter()
                .filter(|&&v| v)
                .count(),
            16
        );
        assert!(tatami_floor_mask(map, &c, &g, [1, 0], &[])
            .iter()
            .all(|v| !*v));
        for i in 0..16 {
            let mut bad = t.clone();
            bad[i].source.tile_index ^= 1;
            let c: Vec<_> = bad.iter().collect();
            assert!(tatami_floor_mask(map, &c, &g, [0, 0], &[])
                .iter()
                .all(|v| !*v));
            let mut excluded = vec![false; 16];
            excluded[i] = true;
            assert!(
                tatami_floor_mask(map, &t.iter().collect::<Vec<_>>(), &g, [0, 0], &excluded)
                    .iter()
                    .all(|v| !*v)
            );
        }
        for mode in 0..4 {
            let mut mesh = TerrainMeshData::default();
            append_top(
                &mut mesh.textured,
                g.bounds(3, 0).into(),
                if mode == 0 { 8. } else { 0. },
                if mode == 1 { g.uv(1, 0) } else { g.uv(3, 0) },
            );
            if mode == 2 {
                mesh.textured.uvs[0][0] += 0.02;
            }
            if mode == 3 {
                mesh.textured.normals[0] = [0., 0., 1.];
            }
            let before = mesh.textured.clone();
            assert_eq!(
                modeled_interiors::finish_surfaces(&mut mesh, map, &c, &g, [0, 0]),
                0
            );
            assert_eq!(mesh.textured.positions, before.positions);
            assert_eq!(mesh.textured.indices, before.indices);
        }
    }
    let (t, g) = floor_fixture();
    let c: Vec<_> = t.iter().collect();
    for map in [
        "UnreviewedTraditionalHouse",
        "KurtsHouseAnnex",
        "Route39Barn",
        "WiseTriosRoom",
        "MountMoonGiftShop",
    ] {
        assert!(!floor_map(map));
        assert!(tatami_floor_mask(map, &c, &g, [0, 0], &[])
            .iter()
            .all(|v| !v));
    }
}
fn theater_fixture() -> (Vec<VisualTile>, GridGeometry) {
    let g = GridGeometry {
        width: 24,
        height: 12,
        tile_width: 8.,
        tile_height: 8.,
        origin_x: -96.,
        origin_z: -48.,
    };
    let tatami = [
        [0x44, 0x45, 0x45, 0x46],
        [0x54, 0x55, 0x55, 0x56],
        [0x45, 0x46, 0x44, 0x45],
        [0x55, 0x56, 0x54, 0x55],
    ];
    let mut t = Vec::new();
    for y in 0..12 {
        for x in 0..24 {
            let b = if y < 4 {
                0x2d
            } else if y < 8 {
                0x2c
            } else if y < 10 {
                if x < 4 {
                    0x2e
                } else if x >= 20 {
                    0x2f
                } else {
                    0x30
                }
            } else {
                4
            };
            let a = match y {
                0 => {
                    if x % 2 == 0 {
                        0x4e
                    } else {
                        0x4f
                    }
                }
                1 => 0x5e,
                2..=8 => 0x50,
                9 => {
                    if matches!(x, 2 | 3 | 20 | 21) {
                        0x5f
                    } else {
                        0x0f
                    }
                }
                _ => tatami[y % 4][x % 4],
            };
            t.push(tile(x, y, b, a));
        }
    }
    (t, g)
}
#[test]
fn theater_external_pack_scene_has_no_interior_stage_overlap() {
    let path=std::path::PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
        .unwrap_or_else(||"content-packs/core-modular.browser.crystalpack".into()));
    if !path.exists() {return;}
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
    let map=&loaded.pack().data().maps["DanceTheater"];
    let layout=&loaded.pack().runtime_files()["data/tilesets/traditional_house_metatiles.bin"];
    let w=map.attributes.width as usize*4;let h=map.attributes.height as usize*4;
    let sources=(0..w*h).map(|i| {let x=i%w;let y=i/w;
        let b=map.blocks[y/4*(w/4)+x/4]as u16;
        VisualTileSource{tileset_id:"traditional_house".into(),metatile_id:b,
            subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,
            tile_index:layout[b as usize*16+y%4*4+x%4]as u16}
    }).collect();
    let mut frame=super::super::tests::frame(w as u32,h as u32,sources);
    frame.map_id="DanceTheater".into();
    frame.jump_ledges=vec![crystal_render_api::VisualJumpLedge {
        start:bevy::prelude::IVec2::new(0,8),end:bevy::prelude::IVec2::new(w as i32,8),
        descent:bevy::prelude::IVec2::Y,
    }].into();
    let mut samples=TerrainImageSamples::default();
    for tile in &frame.tiles {samples.pixels.insert(tile.texture.id(),
        TileImageSample::Rgba([128,100,20,255].repeat(64)));}
    let profiles=crate::live_profiles::LiveProfiles::default();
    let mesh=build_terrain_mesh_internal(&frame,Some(&samples),true,Some(&profiles.document),None,None).unwrap();
    let north=-(h as f32)*4.;
    for (label,surface) in [("textured",&mesh.textured),("solid",&mesh.solid)] {
        for tri in surface.indices.chunks_exact(3) {
            let p=tri.iter().map(|&i|surface.positions[i as usize]).collect::<Vec<_>>();
            if p.iter().all(|v|v[0].abs()<80.&&v[2]>north+16.&&v[2]<north+79.9) {
                assert!(p.iter().all(|v|v[1]<=8.0001),"{label} geometry above stage {p:?}");
            }
        }
    }
}
#[test]
fn theater_full_scene_retains_level_cap_without_interior_crossing_faces() {
    let (tiles,g)=theater_fixture();
    let mut frame=super::super::tests::frame(g.width as u32,g.height as u32,
        tiles.iter().map(|t|t.source.clone()).collect());
    frame.map_id="DanceTheater".into();
    let mut samples=TerrainImageSamples::default();
    for tile in &frame.tiles {samples.pixels.insert(tile.texture.id(),
        TileImageSample::Rgba([128,100,20,255].repeat(64)));}
    let profiles=crate::live_profiles::LiveProfiles::default();
    let mesh=build_terrain_mesh_internal(&frame,Some(&samples),true,Some(&profiles.document),None,None).unwrap();
    for (label,surface) in [("textured",&mesh.textured),("solid",&mesh.solid)] {
        for tri in surface.indices.chunks_exact(3) {
            let points=tri.iter().map(|&i|surface.positions[i as usize]).collect::<Vec<_>>();
            let x=points.iter().map(|p|p[0]).sum::<f32>()/3.;
            if x.abs()<80.&&points.iter().all(|p|p[2]>-24.&&p[2]<31.) {
                assert!(points.iter().all(|p|p[1]>=7.9749)||points.iter().all(|p|p[1]<=0.001),
                    "{label} crossing stage face {points:?}");
                assert!(points.iter().all(|p|p[1]<=8.0001),
                    "{label} extra geometry above stage cap {points:?}");
            }
        }
    }
}
#[test]
fn theater_atomic_ownership_keeps_native_footing_and_floor_accounting() {
    let (t, g) = theater_fixture();
    let c: Vec<_> = t.iter().collect();
    let out = resolve("DanceTheater", &c, &g, [0, 0], None, &vec![false; t.len()]);
    assert_eq!(out.len(), 1);
    let mut mesh = TerrainMeshData::default();
    mesh.authored_cells = vec![None; t.len()];
    mesh.footing_heights = (0..t.len())
        .map(|i| if i / g.width < 10 { 8. } else { 0. })
        .collect();
    let footing = mesh.footing_heights.clone();
    let mut claimed = vec![false; t.len()];
    assert!(append(&mut mesh, &c, &g, &out[0], &mut claimed));
    assert_eq!(claimed.iter().filter(|&&v| v).count(), 240);
    assert_eq!(
        mesh.authored_cells.iter().filter(|v| v.is_some()).count(),
        72
    );
    assert_eq!(mesh.footing_heights, footing);
    assert!(!mesh.textured.positions.is_empty());
    assert!(mesh.solid.cutaway_ranges.is_empty());
    assert!(mesh.solid.positions.iter().chain(&mesh.textured.positions).all(|p| p[0] >= -96.001
        && p[0] <= 96.001
        && p[2] >= -48.001
        && p[2] <= 32.001
        && p[1] >= 0.
        && p[1] <= 24.001));
    // Actor-facing floor rows retain a single, exactly 8px visible top datum.
    assert!(mesh
        .textured
        .positions
        .iter()
        .filter(|p| p[2] > -32. && p[1] > 7.81)
        .all(|p| p[1]<=8.0001&&p[1]>=7.9749));
    assert!(mesh.textured.positions.iter().any(|p|(p[1]-8.).abs()<0.0001));
    for tri in mesh.textured.indices.chunks_exact(3) {
        let points=tri.iter().map(|&i|mesh.textured.positions[i as usize]).collect::<Vec<_>>();
        if points.iter().all(|p|p[2]>-32.&&p[2]<24.) {
            assert!(points.iter().all(|p|p[1]>=7.9749)||points.iter().all(|p|p[1]<=0.001),
                "interior stage cap must not retain crossing side faces: {points:?}");
        }
    }
    for tri in mesh.textured.indices.chunks_exact(3) {
        if tri.iter().all(|&i|mesh.textured.normals[i as usize][1]>0.5) {
            assert!(tri.iter().all(|&i|mesh.textured.positions[i as usize][1]>=7.9749));
        }
    }
    let before = mesh.solid.positions.len();
    assert!(!append(&mut mesh, &c, &g, &out[0], &mut claimed));
    assert_eq!(mesh.solid.positions.len(), before);
    let mut stale = t.clone();
    stale[9 * 24 + 2].source.tile_index = 0x0f;
    let mut empty = TerrainMeshData::default();
    assert!(!append(
        &mut empty,
        &stale.iter().collect::<Vec<_>>(),
        &g,
        &out[0],
        &mut vec![false; t.len()]
    ));
    assert!(empty.solid.indices.is_empty());
}
#[test]
fn custom_stage_profile_has_first_refusal_even_with_same_native_art() {
    let (t, g) = theater_fixture();
    let c: Vec<_> = t.iter().collect();
    let custom:Document=serde_json::from_str(r#"{"objects":[{"name":"My stage screen","map":"DanceTheater","tileset":"traditional_house","metatile":45,"origin":[0,0],"tiles":[[78,79],[94,94]],"ground":69,"top_pixels":8,"depth_pixels":4}]}"#).unwrap();
    assert!(resolve(
        "DanceTheater",
        &c,
        &g,
        [0, 0],
        Some(&custom),
        &vec![false; t.len()]
    )
    .is_empty());
}
