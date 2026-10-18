//! Divider faces must reach their cap without changing the authored openings.
use super::*;
use std::collections::BTreeMap;

fn assert_closed_sides(mesh: &SurfaceMeshData, height: f32) {
    let mut faces: BTreeMap<[i32; 6], Vec<(i32, i32)>> = BTreeMap::new();
    for (points, normals) in mesh
        .positions
        .chunks_exact(4)
        .zip(mesh.normals.chunks_exact(4))
    {
        if normals[0][1].abs() > 0.1 {
            continue;
        }
        let low = [0, 1, 2].map(|a| points.iter().map(|p| p[a]).fold(f32::INFINITY, f32::min));
        let high = [0, 1, 2].map(|a| {
            points
                .iter()
                .map(|p| p[a])
                .fold(f32::NEG_INFINITY, f32::max)
        });
        let key = [
            normals[0][0],
            normals[0][2],
            low[0],
            high[0],
            low[2],
            high[2],
        ]
        .map(|v| (v * 1000.).round() as i32);
        faces.entry(key).or_default().push((
            (low[1] * 1000.).round() as i32,
            (high[1] * 1000.).round() as i32,
        ));
    }
    assert!(!faces.is_empty());
    for spans in faces.values_mut() {
        spans.sort();
        assert_eq!(
            spans,
            &vec![(0, (height * 500.).round() as i32),
                ((height * 500.).round() as i32, (height * 1000.).round() as i32)],
            "both outward sides run continuously from floor to ceiling cap"
        );
    }
}

#[test]
fn external_power_plant_facility_divider_faces_reach_caps_and_preserve_openings() {
    let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("completed external pack");
    let loaded =
        crystal_assets::read_loaded_verified_compiled_game_pack(std::path::Path::new(&path))
            .unwrap();
    let pack = loaded.pack();
    let map = &pack.data().maps["PowerPlant"];
    let art = &pack.runtime_files()["data/tilesets/facility_metatiles.bin"];
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
                    tileset_id: "facility".into(),
                    metatile_id: block,
                    subtile_column: (x % 4) as u8,
                    subtile_row: (y % 4) as u8,
                    tile_index: art[block as usize * 16 + y % 4 * 4 + x % 4] as u16,
                },
            }
        })
        .collect::<Vec<_>>();
    let cells = tiles.iter().collect::<Vec<_>>();
    // Include the actual native32-unit tile scale, not only source pixels.
    for tile_size in [4., 8., 16., 32.] {
    let g = GridGeometry {
        width: w,
        height: h,
        tile_width: tile_size,
        tile_height: tile_size,
        origin_x: 0.,
        origin_z: 0.,
    };
    let mut expected = vec![false; w * h];
    for p in facility_divider_placements("PowerPlant", &cells, &g) {
        for x in p.column..p.column + p.width {
            expected[p.row * w + x] = true;
            expected[(p.row + 1) * w + x] = true;
        }
    }
    for p in facility_vertical_divider_placements("PowerPlant", &cells, &g) {
        for y in p.row..p.row + p.height {
            expected[y * w + p.column] = true;
            expected[y * w + p.column + 1] = true;
        }
    }
    for p in facility_divider_cap_placements("PowerPlant", &cells, &g) {
        for x in p.column..p.column + p.width {
            expected[p.row * w + x] = true;
        }
    }
    assert!(expected.iter().any(|v| *v));
    let source_before = tiles.iter().map(|t| t.source.clone()).collect::<Vec<_>>();
    let mut mesh = TerrainMeshData {
        footing_heights: vec![3.25; w * h],
        ..Default::default()
    };
    let mut claimed = vec![false; w * h];
    append_facility_divider_network(&mut mesh, "PowerPlant", &cells, &g, &mut claimed).unwrap();
    assert_eq!(
        claimed, expected,
        "wall footprints and every source doorway stay exact"
    );
    assert_eq!(mesh.footing_heights, vec![3.25; w * h]);
    assert_eq!(
        tiles.iter().map(|t| t.source.clone()).collect::<Vec<_>>(),
        source_before
    );
    assert_closed_sides(&mesh.textured, 4. * tile_size);
    assert_eq!(mesh.textured.cutaway_ranges, mesh.layered_alpha_textured_ranges);
    assert!(mesh.depth_coverage_textured_ranges.is_empty(), "smooth divider reveal must not acquire the cabinet pixel mask");
    for (quad, (points, normals)) in mesh.textured.positions.chunks_exact(4)
        .zip(mesh.textured.normals.chunks_exact(4)).enumerate() {
        let floor = normals[0][1] > 0.99 && points.iter().all(|p| p[1] == 0.);
        for i in quad*4..quad*4+4 {
            assert_eq!(mesh.textured.cutaway_ranges.iter().any(|r| r.contains(&i)), !floor,
                "wall faces/caps reveal together; source floor stays opaque");
        }
    }

    let caps = mesh
        .textured
        .positions
        .chunks_exact(4)
        .zip(mesh.textured.normals.chunks_exact(4))
        .filter(|(p, n)| n[0][1] > 0.99 && p.iter().all(|p| p[1] == 4. * tile_size))
        .count();
    assert_eq!(caps, expected.iter().filter(|v| **v).count());
    }
}

#[test]
fn facility_divider_unreviewed_maps_remain_unclaimed() {
    let g = GridGeometry {
        width: 1,
        height: 1,
        tile_width: 8.,
        tile_height: 8.,
        origin_x: 0.,
        origin_z: 0.,
    };
    let tile = VisualTile {
        column: 0,
        row: 0,
        texture: Default::default(),
        animation_frames: None,
        priority: false,
        source: VisualTileSource {
            tileset_id: "facility".into(),
            metatile_id: 0,
            subtile_column: 0,
            subtile_row: 0,
            tile_index: 0x41,
        },
    };
    let mut mesh = TerrainMeshData::default();
    let mut claimed = vec![false];
    append_facility_divider_network(&mut mesh, "PlayersHouse1F", &[&tile], &g, &mut claimed)
        .unwrap();
    assert!(mesh.textured.positions.is_empty());
    assert_eq!(claimed, vec![false]);
}

#[test]
fn trainer_reception_counter_stays_low_while_arena_walls_reach_the_scaled_ceiling() {
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap()).unwrap();
    let pack=loaded.pack();let map=&pack.data().maps["TrainerHouseB1F"];
    let art=&pack.runtime_files()["data/tilesets/facility_metatiles.bin"];
    let (w,h)=(map.attributes.width as usize*4,map.attributes.height as usize*4);
    let tiles=(0..w*h).map(|i| {
        let (x,y)=(i%w,i/w);let block=map.blocks[y/4*(w/4)+x/4];
        VisualTile {column:x as u32,row:y as u32,texture:Default::default(),animation_frames:None,priority:false,
            source:VisualTileSource {tileset_id:"facility".into(),metatile_id:block,subtile_column:(x%4) as u8,
                subtile_row:(y%4) as u8,tile_index:art[block as usize*16+y%4*4+x%4] as u16}}
    }).collect::<Vec<_>>();
    let cells=tiles.iter().collect::<Vec<_>>();
    for size in [8.,32.] {
        let g=GridGeometry {width:w,height:h,tile_width:size,tile_height:size,origin_x:0.,origin_z:0.};
        let mut mesh=TerrainMeshData {footing_heights:vec![3.25;w*h],..Default::default()};
        let mut claimed=vec![false;w*h];
        append_facility_divider_network(&mut mesh,"TrainerHouseB1F",&cells,&g,&mut claimed).unwrap();
        let mut counters=0;let mut walls=0;let mut cabinet_caps=0;
        for (quad,normals) in mesh.textured.positions.chunks_exact(4).zip(mesh.textured.normals.chunks_exact(4)) {
            if normals[0][1]<0.99 || quad[0][1]==0. {continue;}
            let x=(quad.iter().map(|p|p[0]).fold(f32::INFINITY,f32::min)/size).round() as usize;
            let y=(quad.iter().map(|p|p[2]).fold(f32::INFINITY,f32::min)/size).round() as usize;
            let counter=((12..14).contains(&x)&&(1..4).contains(&y))
                ||((12..20).contains(&x)&&(4..6).contains(&y));
            assert!(quad.iter().all(|p|p[1]==if counter {size}else{4.*size}),
                "receptionist counter stays below the trainer; arena walls meet the ceiling at ({x},{y})");
            if counter {counters+=1;}else{walls+=1;}
            if tiles[y*w+x].source.metatile_id==2 {cabinet_caps+=1;}
        }
        // This direct legacy-builder fixture includes eight raw cabinet
        // header cells. Production retires them under the authored cabinets;
        // they are not additional arena walls or a changed source footprint.
        assert_eq!(cabinet_caps,8);
        assert_eq!((counters,walls),(22,148));
        assert_eq!(claimed.iter().filter(|&&x|x).count(),170);
        assert_eq!(mesh.footing_heights,vec![3.25;w*h]);
        let (pw,ph)=(w+8,h+8);
        let mut border=tiles[0].clone();border.source.metatile_id=u16::MAX;border.source.tile_index=u16::MAX;
        let mut padded=vec![border;pw*ph];
        for y in 0..h {for x in 0..w {padded[(y+4)*pw+x+4]=tiles[y*w+x].clone();}}
        let pg=GridGeometry {width:pw,height:ph,origin_x:-4.*size,origin_z:-4.*size,..g};
        let mut shifted=TerrainMeshData::default();let mut padded_claims=vec![false;pw*ph];
        append_facility_divider_network(&mut shifted,"TrainerHouseB1F",&padded.iter().collect::<Vec<_>>(),&pg,&mut padded_claims).unwrap();
        assert_eq!(shifted.textured.positions,mesh.textured.positions,
            "source topology preserves counter and arena heights through padded render grids");
        assert_eq!(shifted.textured.cutaway_ranges,mesh.textured.cutaway_ranges);
    }
}
