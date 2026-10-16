fn workbench_fixture() -> (Vec<VisualTile>, GridGeometry) {
    let art = [
        [0x02, 0x03, 0x04, 0x05],
        [0x12, 0x13, 0x14, 0x15],
        [0x06, 0x07, 0x08, 0x09],
        [0x26, 0x01, 0x26, 0x01],
    ];
    let tiles = (0..16)
        .map(|i| VisualTile {
            animation_frames: None,
            column: (i % 4) as u32,
            row: (i / 4) as u32,
            source: VisualTileSource {
                tileset_id: Arc::from("facility"),
                metatile_id: if i < 12 { 0x28 } else { 0x07 },
                subtile_column: (i % 4) as u8,
                subtile_row: (i / 4) as u8,
                tile_index: art[i / 4][i % 4],
            },
            texture: Default::default(),
            priority: false,
        })
        .collect();
    let g = GridGeometry {
        width: 4,
        height: 4,
        tile_width: 8.,
        tile_height: 8.,
        origin_x: 0.,
        origin_z: 0.,
    };
    (tiles, g)
}
#[test]
fn complete_workbench_keeps_native_checker_floor_and_footing_and_live_readout() {
    let (tiles, g) = workbench_fixture();
    let cells: Vec<_> = tiles.iter().collect();
    let mut claimed = vec![false; 16];
    let out = resolve("MrPokemonsHouse", &cells, &g, [0, 0], None, &claimed);
    assert_eq!(out.len(), 1);
    let mut mesh = TerrainMeshData::default();
    mesh.authored_cells = vec![None; 16];
    mesh.footing_heights = (0..16).map(|i| i as f32 * 0.01).collect();
    let footing = mesh.footing_heights.clone();
    assert!(append(&mut mesh, &cells, &g, &out[0], &mut claimed));
    assert_eq!(
        mesh.footing_heights, footing,
        "furniture does not edit source support heights"
    );
    assert_eq!(claimed.iter().filter(|&&v| v).count(), 12);
    assert!(
        claimed[12..].iter().all(|&v| !v),
        "genuine lower source floor remains unclaimed"
    );
    assert_eq!(
        mesh.authored_cells.iter().filter(|v| v.is_some()).count(),
        12
    );
    assert!(mesh.textured.positions[..48].iter().all(|p| p[1] == 0.));
    for y in 0..3 {
        for x in 0..4 {
            let i = y * 4 + x;
            let sample = out[0].resolved.ground_for(x, y);
            assert_eq!(
                cells[sample].source.tile_index,
                if (x + y) % 2 == 0 { 1 } else { 0x26 }
            );
            let uv = g.uv(sample % 4, sample / 4);
            assert!(
                mesh.textured.uvs[i * 4..i * 4 + 4]
                    .iter()
                    .all(|p| p[0] >= uv.0 && p[0] <= uv.1 && p[1] >= uv.2 && p[1] <= uv.3)
            );
        }
    }
    assert!(
        mesh.textured.positions[48..]
            .iter()
            .all(|p| p[1] > 11. && p[1] < 17. && p[2] > 14.19 && p[2] < 14.21)
    );
    assert!(mesh.solid.positions.iter().all(|p| p[0] >= 0.
        && p[0] <= 32.
        && p[1] >= 0.
        && p[1] <= 18.
        && p[2] >= 6.
        && p[2] <= 24.));
    assert!(mesh.solid.cutaway_ranges.is_empty() && mesh.textured.cutaway_ranges.is_empty());
    let before = mesh.clone();
    assert!(!append(&mut mesh, &cells, &g, &out[0], &mut claimed));
    assert_eq!(mesh, before);
}
#[test]
fn custom_workbench_profile_wins_and_stale_source_or_floor_appends_nothing() {
    let (mut tiles, g) = workbench_fixture();
    let empty = vec![false; 16];
    let profile:Document=serde_json::from_str(r#"{"objects":[{"name":"My offset instrument bench","map":"MrPokemonsHouse","tileset":"facility","metatile":40,"origin":[0,0],"tiles":[[2,3,4,5],[18,19,20,21],[6,7,8,9]],"ground":1,"top_pixels":8,"depth_pixels":18}]}"#).unwrap();
    assert!(
        resolve(
            "MrPokemonsHouse",
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            [0, 0],
            Some(&profile),
            &empty
        )
        .is_empty()
    );
    let out = resolve(
        "MrPokemonsHouse",
        &tiles.iter().collect::<Vec<_>>(),
        &g,
        [0, 0],
        None,
        &empty,
    );
    assert_eq!(out.len(), 1);
    for i in [
        0,
        5,
        10,
        out[0].resolved.ground[0],
        out[0].resolved.ground[1],
    ] {
        let old = tiles[i].source.tile_index;
        tiles[i].source.tile_index ^= 1;
        let mut mesh = TerrainMeshData::default();
        let mut claimed = empty.clone();
        assert!(!append(
            &mut mesh,
            &tiles.iter().collect::<Vec<_>>(),
            &g,
            &out[0],
            &mut claimed
        ));
        assert_eq!(mesh, TerrainMeshData::default());
        assert_eq!(claimed, empty);
        tiles[i].source.tile_index = old;
    }
}
#[test]
fn six_runtime_models_fit_finite_authored_geometry_and_preserve_the_staff_openings() {
    for (kind, index) in [
        (Kind::Workbench, 0),
        (Kind::PhoneDesk, 1),
        (Kind::MemoDesk, 2),
        (Kind::ReceptionU, 3),
        (Kind::ReceptionL, 4),
        (Kind::CounterExtension, 5),
    ] {
        let (bounds, rise) = kind.fitting();
        let mut mesh = SurfaceMeshData::default();
        model(index).append_fitted(&mut mesh, bounds, 0., rise);
        assert!(mesh.indices.len() > 100 && mesh.indices.len() < 7000);
        assert!(
            mesh.positions
                .iter()
                .all(|p| p.iter().all(|v| v.is_finite())
                    && p[0] >= bounds[0] - 0.001
                    && p[0] <= bounds[1] + 0.001
                    && p[1] >= -0.001
                    && p[1] <= rise + 0.001
                    && p[2] >= bounds[2] - 0.001
                    && p[2] <= bounds[3] + 0.001)
        );
        assert!(
            mesh.normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
        );
        if kind == Kind::ReceptionL {
            assert!(
                mesh.positions
                    .iter()
                    .all(|p| p[0] <= 16.001 || p[2] <= 16.001)
            );
        }
        if kind == Kind::ReceptionU {
            assert!(mesh.positions.iter().all(|p| p[0] <= 16.001
                || p[0] >= 143.999
                || p[2] <= 16.001
                || (p[0] >= 79.999 && p[0] <= 96.001 && p[2] <= 24.001)));
        }
        assert!(
            mesh.colors
                .iter()
                .all(|c| c.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)))
        );
    }
}
#[test]
fn live_display_triangles_face_outward_and_remain_inside_the_live_source_drawing() {
    let (tiles, g) = workbench_fixture();
    let cells: Vec<_> = tiles.iter().collect();
    let mut claimed = vec![false; 16];
    let out = resolve("MrPokemonsHouse", &cells, &g, [0, 0], None, &claimed);
    let mut mesh = TerrainMeshData::default();
    assert!(append(&mut mesh, &cells, &g, &out[0], &mut claimed));
    for ids in mesh.textured.indices.chunks_exact(3) {
        let a = Vec3::from_array(mesh.textured.positions[ids[0] as usize]);
        let b = Vec3::from_array(mesh.textured.positions[ids[1] as usize]);
        let c = Vec3::from_array(mesh.textured.positions[ids[2] as usize]);
        let normal = Vec3::from_array(mesh.textured.normals[ids[0] as usize]);
        assert!((b - a).cross(c - a).dot(normal) > 0.);
    }
    for uv in &mesh.textured.uvs[48..] {
        assert!(uv[0] >= 0. && uv[0] <= 0.5 && uv[1] >= 0. && uv[1] <= 0.25);
    }
}

fn round_stool_fixture() -> (Vec<VisualTile>, GridGeometry) {
    let g = GridGeometry { width: 6, height: 6, tile_width: 8., tile_height: 8., origin_x: -8., origin_z: 13. };
    let mut cells: Vec<_> = (0..36).map(|i| VisualTile {
        animation_frames: None,
        column: (i % 6) as u32, row: (i / 6) as u32, texture: Default::default(), priority: false,
        source: VisualTileSource { tileset_id: Arc::from("radio_tower"), metatile_id: 1,
            subtile_column: (i % 6 % 4) as u8, subtile_row: (i / 6 % 4) as u8, tile_index: 1 },
    }).collect();
    for y in 0..2 { for x in 0..2 {
        cells[(2 + y) * 6 + 2 + x].source = VisualTileSource {
            tileset_id: Arc::from("radio_tower"), metatile_id: 0x27,
            subtile_column: (2 + x) as u8, subtile_row: y as u8,
            tile_index: [[0x2c, 0x2d], [0x3c, 0x3d]][y][x],
        };
    }}
    (cells, g)
}
#[test]
fn radio_round_stool_is_complete_source_contained_and_keeps_native_footing() {
    let (mut cells, g) = round_stool_fixture();
    let empty = vec![false; cells.len()];
    for map in ["RadioTower3F", "RadioTower4F", "RadioTower5F"] {
        let out = resolve(map, &cells.iter().collect::<Vec<_>>(), &g, [0,0], None, &empty);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].resolved.kind(), Kind::RoundStool);
        let mut mesh = TerrainMeshData::default();
        mesh.footing_heights = (0..cells.len()).map(|i| i as f32 * 0.02).collect();
        let footing = mesh.footing_heights.clone();
        let mut claimed = empty.clone();
        assert!(append(&mut mesh, &cells.iter().collect::<Vec<_>>(), &g, &out[0], &mut claimed));
        assert_eq!(mesh.footing_heights, footing);
        assert_eq!(claimed.iter().filter(|&&v| v).count(), 4);
        let (w, _, n, _) = g.bounds(2, 2);
        let mut cached = SurfaceMeshData::default();
        crate::interior_models::model(ModelKind::ArcadeStool)
            .append_fitted(&mut cached, [w + 2., w + 14., n, n + 12.], 0., 7.);
        assert!(mesh.solid.indices.is_empty());
        let first=out[0].indices(g.width).count()*4;
        assert_eq!(mesh.textured.positions[first..],cached.positions);
        assert_eq!(mesh.textured.normals[first..],cached.normals);
        assert_eq!(mesh.textured.indices[out[0].indices(g.width).count()*6..].iter()
            .map(|i|i-first as u32).collect::<Vec<_>>(),cached.indices);
        let top=pixel_uv(&g,&out[0].resolved,8.5,5.5);
        let side=pixel_uv(&g,&out[0].resolved,3.5,9.5);
        for (n,uv) in mesh.textured.normals[first..].iter().zip(&mesh.textured.uvs[first..]) {
            assert_eq!(*uv,if n[1]>0.5 {top}else{side});
        }
        for y in 0..2 { for x in 0..2 {
            let i = (2 + y) * g.width + 2 + x;
            cells[i].source.tile_index ^= 1;
            assert!(resolve(map, &cells.iter().collect::<Vec<_>>(), &g, [0,0], None, &empty).is_empty());
            let mut stale = TerrainMeshData::default();
            let mut fresh = empty.clone();
            assert!(!append(&mut stale, &cells.iter().collect::<Vec<_>>(), &g, &out[0], &mut fresh));
            assert_eq!(stale, TerrainMeshData::default());
            assert_eq!(fresh, empty);
            cells[i].source.tile_index ^= 1;
        }}
    }
    assert!(resolve("RadioTower3FBeta", &cells.iter().collect::<Vec<_>>(), &g, [0,0], None, &empty).is_empty());
    let custom: Document = serde_json::from_str(r#"{"objects":[{"name":"My round stool","map":"RadioTower3F","tileset":"radio_tower","metatile":39,"origin":[2,0],"tiles":[[44,45],[60,61]],"ground":1,"top_pixels":8,"depth_pixels":12}]}"#).unwrap();
    assert!(resolve("RadioTower3F", &cells.iter().collect::<Vec<_>>(), &g, [0,0], Some(&custom), &empty).is_empty());
}

#[test]
fn kanto_basement_instrument_lid_is_horizontal_and_controls_fill_the_front() {
    let (mut tiles,g)=workbench_fixture();
    for y in 0..4 {for x in 0..4 {
        let s=&mut tiles[y*4+x].source;
        s.metatile_id=if x<2 {2}else{7};
        s.tile_index=if x<2 {[[64,66],[10,11],[10,11],[26,27]][y][x]} else if (x+y)%2==0 {1}else{38};
    }}
    let cells=tiles.iter().collect::<Vec<_>>();let mut claimed=vec![false;16];
    let placements=resolve("TrainerHouseB1F",&cells,&g,[0,0],None,&claimed);
    let p=placements.iter().find(|p|p.resolved.kind()==Kind::InstrumentBank).unwrap();
    let mut mesh=TerrainMeshData{footing_heights:vec![1.25;16],authored_cells:vec![None;16],..Default::default()};
    assert!(append(&mut mesh,&cells,&g,p,&mut claimed));
    assert_eq!(mesh.footing_heights,vec![1.25;16]);assert_eq!(claimed.iter().filter(|&&x|x).count(),8);
    assert!(mesh.solid.indices.is_empty());
    let lid=mesh.textured.positions.iter().zip(&mesh.textured.normals).zip(&mesh.textured.uvs)
        .filter(|((p,n),_)|p[1]>24.&&*n==&[0.,1.,0.]).collect::<Vec<_>>();
    assert_eq!(lid.len(),8);assert!(lid.iter().all(|(_,uv)|uv[1]<=0.25));
    let front=mesh.textured.positions.iter().zip(&mesh.textured.normals).zip(&mesh.textured.uvs)
        .filter(|((p,n),_)|p[2]>32.&&*n==&[0.,0.,1.]).collect::<Vec<_>>();
    assert_eq!(front.len(),24);assert!(front.iter().all(|(_,uv)|uv[1]>=0.25));
    assert!(front.iter().any(|((p,_),_)|p[1]==24.));assert!(front.iter().any(|((p,_),_)|p[1]==0.));
    for i in p.indices(g.width) {let mut changed=tiles.clone();changed[i].source.tile_index^=128;
        assert!(!resolve("TrainerHouseB1F",&changed.iter().collect::<Vec<_>>(),&g,[0,0],None,&[false;16])
            .iter().any(|p|p.resolved.kind()==Kind::InstrumentBank));}
}

#[test]
fn radio_native_materials_reject_renamed_missing_and_unknown_parts_without_appending() {
    let original:serde_json::Value=crate::model_storage::parse(
        crate::model_storage::include_model!("models/facility_radio/radio_phone_desk.mesh.json")).unwrap();
    for mode in 0..3 {
        let mut changed=original.clone();
        match mode {
            0=>changed["name"]="Custom desk".into(),
            1=>changed["primitives"][0]["name"]="Unclassified new material part".into(),
            _=>changed["primitives"].as_array_mut().unwrap().retain(|p|
                !p["name"].as_str().unwrap().starts_with("Telephone ")),
        }
        let text=changed.to_string();let model=crate::interior_models::Model::parse(text.as_str()).unwrap();
        let mut mesh=SurfaceMeshData::default();let before=mesh.clone();
        assert!(!model.append_native_radio(&mut mesh,[0.,32.,0.,16.],14.1,1.,
            |part|radio_affine(Kind::PhoneDesk,part),[[0.1;2];4]));
        assert_eq!(mesh,before);
    }
}

fn radio_top_covers(mesh:&SurfaceMeshData,point:[f32;2])->bool {
    mesh.indices.chunks_exact(3).any(|tri| {
        let p=[0,1,2].map(|j|mesh.positions[tri[j]as usize]);
        if mesh.normals[tri[0]as usize][1]<0.99||p.iter().any(|v|(v[1]-8.).abs()>0.001){return false;}
        let edge=|a:[f32;3],b:[f32;3]|(b[0]-a[0])*(point[1]-a[2])-(b[2]-a[2])*(point[0]-a[0]);
        let sides=[edge(p[0],p[1]),edge(p[1],p[2]),edge(p[2],p[0])];
        sides.iter().all(|s|*s>=-0.001)||sides.iter().all(|s|*s<=0.001)
    })
}

#[test]
fn external_radio_native_counters_and_desks_keep_eight_pixel_tops_source_palette_and_staff_space() {
    let explicit=std::env::var_os("CRYSTAL_RENDER_TEST_PACK");
    let path=explicit.as_ref().map(std::path::PathBuf::from).unwrap_or_else(||
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
    assert!(explicit.is_none()||path.is_file());if !path.is_file(){return;}
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();let pack=loaded.pack();
    let art=&pack.runtime_files()["data/tilesets/radio_tower_metatiles.bin"];
    let png=image::load_from_memory(&pack.runtime_files()["gfx/tilesets/radio_tower.png"]).unwrap().to_luma8();
    let mut counts=[0usize;5];let mut stool_counts=[0usize;2];
    for map in ["RadioTower1F","RadioTower3F","RadioTower4F","RadioTower5F","LavRadioTower1F"] {
        let data=&pack.data().maps[map];let (w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
        let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let tiles=(0..w*h).map(|i| {
            let (x,y)=(i%w,i/w);let b=data.blocks[y/4*(w/4)+x/4];
            VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                source:VisualTileSource{tileset_id:"radio_tower".into(),metatile_id:b,subtile_column:(x%4)as u8,
                    subtile_row:(y%4)as u8,tile_index:art[b as usize*16+y%4*4+x%4]as u16}}
        }).collect::<Vec<_>>();
        let refs=tiles.iter().collect::<Vec<_>>();let empty=vec![false;w*h];
        for p in resolve(map,&refs,&g,[0,0],None,&empty) {
            let kind=p.resolved.kind();
            if matches!(kind,Kind::Stool|Kind::RoundStool) {
                stool_counts[usize::from(kind==Kind::RoundStool)]+=1;
                let mut mesh=TerrainMeshData{footing_heights:vec![2.5;w*h],..Default::default()};
                let mut claimed=empty.clone();assert!(append(&mut mesh,&refs,&g,&p,&mut claimed));
                assert!(mesh.solid.indices.is_empty());assert_eq!(mesh.footing_heights,vec![2.5;w*h]);
                let y=if kind==Kind::Stool {16}else{0};
                for (pixel,gray) in [([8,y+5],170),([3,y+9],85)] {
                    let s=&tiles[(p.resolved.row+pixel[1]/8)*w+p.resolved.column+pixel[0]/8].source;
                    assert_eq!(png.get_pixel(s.tile_index as u32%16*8+(pixel[0]%8)as u32,
                        s.tile_index as u32/16*8+(pixel[1]%8)as u32).0[0],gray);
                }
                continue;
            }
            let index=match kind {
                Kind::PhoneDesk=>0,Kind::MemoDesk=>1,Kind::ReceptionU=>2,Kind::ReceptionL=>3,
                Kind::CounterExtension=>4,_=>continue,
            };counts[index]+=1;
            let mut claimed=empty.clone();let mut mesh=TerrainMeshData{footing_heights:vec![2.5;w*h],
                authored_cells:vec![None;w*h],..Default::default()};
            assert!(append(&mut mesh,&refs,&g,&p,&mut claimed));
            assert!(mesh.solid.indices.is_empty(),"native furniture cannot keep cream/teal paint");
            assert_eq!(mesh.footing_heights,vec![2.5;w*h]);
            assert_eq!(claimed.iter().filter(|v|**v).count(),p.indices(w).count());
            assert_eq!(mesh.authored_cells.iter().filter(|v|v.is_some()).count(),p.indices(w).count());
            let (west,_,north,_)=g.bounds(p.resolved.column,p.resolved.row);
            let start=p.indices(w).count()*4;
            let front=if matches!(kind,Kind::ReceptionU|Kind::ReceptionL) {[20usize,10]}else{[3,10]};
            let probes=[[3,3],front,[1,1],[0,0]];
            for (pixel,gray) in probes.into_iter().zip([170,85,255,0]) {
                let s=&tiles[(p.resolved.row+pixel[1]/8)*w+p.resolved.column+pixel[0]/8].source;
                assert_eq!(png.get_pixel(s.tile_index as u32%16*8+(pixel[0]%8)as u32,
                    s.tile_index as u32/16*8+(pixel[1]%8)as u32).0[0],gray,
                    "native gray/white/ink roles require their actual source pixels");
                let uv=pixel_uv(&g,&p.resolved,pixel[0]as f32+0.5,pixel[1]as f32+0.5);
                assert!(mesh.textured.uvs[start..].contains(&uv));
            }
            assert!(mesh.textured.positions[start..].iter().all(|v|v[1]>=0.&&v[1]<=if kind==Kind::ReceptionU {18.001}else{11.001}));
            let mut broad_top=false;
            for tri in mesh.textured.indices.chunks_exact(3).filter(|tri|tri.iter().all(|i|*i as usize>=start)) {
                let a=Vec3::from_array(mesh.textured.positions[tri[0]as usize]);
                let b=Vec3::from_array(mesh.textured.positions[tri[1]as usize]);
                let c=Vec3::from_array(mesh.textured.positions[tri[2]as usize]);
                let cross=(b-a).cross(c-a);let n=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);
                assert!(cross.dot(n)>0.);
                if n.y>0.99&&cross.length()*0.5>12. {
                    let terminal=kind==Kind::ReceptionU&&[a,b,c].iter().all(|v|
                        (80.-0.001..=96.001).contains(&(v.x-west))&&(18.-0.001..=24.001).contains(&(v.z-north)));
                    assert!(a.y<=8.001||terminal,"projected front height cannot become extra counter height");
                    broad_top|=(a.y-8.).abs()<0.001;
                }
            }
            assert!(broad_top,"{map} must contain a real eight-pixel worktop");
            if matches!(kind,Kind::ReceptionU|Kind::ReceptionL) {
                for z in [12.,18.,24.,32.,40.,47.] {
                    assert!(radio_top_covers(&mesh.textured,[west+8.,north+z]),"west corner/worktop must join");
                    if kind==Kind::ReceptionU {
                        assert!(radio_top_covers(&mesh.textured,[west+152.,north+z]),"east corner/worktop must join");
                    }
                }
                if kind==Kind::ReceptionU {
                    for (pixel,gray) in [([83usize,5usize],0),([84,6],85),([84,8],85)] {
                        let src=&refs[(p.resolved.row+pixel[1]/8)*w+p.resolved.column+pixel[0]/8].source;
                        assert_eq!(png.get_pixel(src.tile_index as u32%16*8+(pixel[0]%8)as u32,
                            src.tile_index as u32/16*8+(pixel[1]%8)as u32).0[0],gray,"actual LCD rows, not white cap");
                    }
                    for pixel in [[83.,5.],[93.,10.],[88.,8.]] {
                        assert!(mesh.textured.uvs.contains(&pixel_uv(&g,&p.resolved,pixel[0],pixel[1])),"LCD art crosses both source rows");
                    }
                    let mut old_gap=SurfaceMeshData::default();
                    assert!(model(3).append_native_radio(&mut old_gap,[0.,160.,0.,48.],18.,1.,|part|
                        if part==RadioFurniturePart::Return {[2./3.,0.,5./6.,8.]}
                        else {radio_affine(kind,part)},[[0.1;2];4]));
                    assert!(!radio_top_covers(&old_gap,[8.,18.]),"coverage check must detect a separated return");
                }
            }
            for v in &mesh.textured.positions[start..] {
                let (x,z)=(v[0]-west,v[2]-north);
                assert!(z>=7.999&&z<=if matches!(kind,Kind::ReceptionU|Kind::ReceptionL) {48.001}else{16.001});
                if kind==Kind::ReceptionL {assert!(x<=16.001||z<=16.001);}
                if kind==Kind::ReceptionU {assert!(x<=16.001||x>=143.999||z<=16.001
                    ||((79.999..=96.001).contains(&x)&&z<=24.001));}
            }
            assert!(mesh.textured.normals.iter().all(|n|(Vec3::from_array(*n).length()-1.).abs()<0.001));
            for warp in &data.events.warps {for dy in 0..2 {for dx in 0..2 {
                assert!(!claimed[(warp.y as usize*2+dy)*w+warp.x as usize*2+dx]);
            }}}
        }
    }
    assert_eq!(counts,[6,3,1,1,1],"every actual source desk/reception variant must be exercised");
    assert_eq!(stool_counts,[3,6],"both guarded operator stool families retain native gray paint");
}
