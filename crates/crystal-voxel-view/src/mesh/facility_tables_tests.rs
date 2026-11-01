fn fixture(kind: Kind) -> (Vec<VisualTile>, GridGeometry, &'static str) {
    let (map, rows, blocks): (&str, Vec<Vec<u16>>, Vec<Vec<u16>>) = match kind {
        Kind::ReceptionCounter => {
            let blocks=(0..8).map(|y|(0..12).map(|x|
                [[0x35,0x35,0x1e],[0x34,0x34,0x3f]][y/4][x/4]).collect::<Vec<_>>()).collect::<Vec<_>>();
            let rows=blocks.iter().enumerate().map(|(y,row)|row.iter().enumerate().map(|(x,b)| {
                let drawing=match b {
                    0x35=>[[34,34,34,34],[34,34,34,34],[14,15,1,38],[30,31,38,1]],
                    0x34=>[[1,38,1,38],[38,1,38,1],[65,65,65,65],[77,77,77,77]],
                    0x1e=>[[34,34,34,34],[34,34,64,66],[1,38,80,82],[38,1,80,82]],
                    _=>[[1,38,80,82],[38,1,80,82],[65,66,80,82],[77,78,26,27]],
                };drawing[y%4][x%4]
            }).collect()).collect();
            ("SilphCo1F",rows,blocks)
        },
        Kind::Sofa => (
            "SilphCo1F",
            vec![vec![0x23,0x24,0x24,0x25],vec![0x57,0x58,0x58,0x59],
                vec![1,38,1,38],vec![38,1,38,1]],vec![vec![0x2d;4];4],
        ),
        Kind::SquareTable => (
            "MrPokemonsHouse",
            vec![
                vec![64, 65, 65, 66],
                vec![80, 81, 81, 82],
                vec![80, 72, 73, 82],
                vec![83, 58, 58, 84],
            ],
            vec![vec![0x0f; 4]; 4],
        ),
        Kind::SideDesk => (
            "RuinsOfAlphResearchCenter",
            vec![
                vec![64, 65, 65, 66],
                vec![80, 72, 73, 82],
                vec![6, 7, 8, 9],
                vec![38, 1, 38, 1],
            ],
            vec![vec![0x29; 4]; 4],
        ),
        Kind::Chair => (
            "MrPokemonsHouse",
            vec![vec![14, 15], vec![30, 31]],
            vec![vec![0x27; 2]; 2],
        ),
        Kind::MeetingTable => (
            "PowerPlant",
            vec![
                vec![1, 38, 14, 15, 1, 38, 14, 15],
                vec![38, 1, 30, 31, 38, 1, 30, 31],
                vec![1, 38, 64, 65, 65, 65, 65, 66],
                vec![38, 1, 80, 81, 81, 81, 81, 82],
                vec![1, 38, 80, 81, 81, 81, 81, 82],
                vec![38, 1, 83, 58, 58, 58, 58, 84],
                vec![1, 38, 14, 15, 1, 38, 14, 15],
                vec![38, 1, 30, 31, 38, 1, 30, 31],
            ],
            (0..8)
                .map(|y| {
                    (0..8)
                        .map(|x| [[0x10, 0x11], [0x14, 0x15]][y / 4][x / 4])
                        .collect()
                })
                .collect(),
        ),
    };
    let w = rows[0].len() + 2;
    let h = rows.len() + 2;
    let g = GridGeometry {
        width: w,
        height: h,
        tile_width: 8.,
        tile_height: 8.,
        origin_x: -13.,
        origin_z: 7.,
    };
    let mut cells: Vec<_> = (0..w * h)
        .map(|i| VisualTile {
            animation_frames: None,
            column: (i % w) as u32,
            row: (i / w) as u32,
            texture: Default::default(),
            priority: false,
            source: VisualTileSource {
                tileset_id: Arc::from("facility"),
                metatile_id: 7,
                subtile_column: (i % w % 4) as u8,
                subtile_row: (i / w % 4) as u8,
                tile_index: if (i % w + i / w) % 2 == 0 { 1 } else { 0x26 },
            },
        })
        .collect();
    for y in 0..rows.len() {
        for x in 0..rows[0].len() {
            cells[y * w + x].source = VisualTileSource {
                tileset_id: Arc::from("facility"),
                metatile_id: blocks[y][x],
                subtile_column: if kind == Kind::Chair {
                    (x + 2) as u8
                } else {
                    (x % 4) as u8
                },
                subtile_row: (y % 4) as u8,
                tile_index: rows[y][x],
            };
        }
    }
    (cells, g, map)
}
#[test]
fn facility_tables_are_complete_finite_and_leave_native_actor_support_alone() {
    for (kind, owned) in [
        (Kind::SquareTable, 16),
        (Kind::MeetingTable, 24),
        (Kind::SideDesk, 12),
        (Kind::Chair, 4),
    ] {
        let (cells, g, map) = fixture(kind);
        let refs: Vec<_> = cells.iter().collect();
        let mut claimed = vec![false; cells.len()];
        let placements = resolve(map, &refs, &g, [0, 0], None, &claimed);
        let p = placements
            .iter()
            .find(|p| p.resolved.kind() == kind)
            .unwrap();
        let mut mesh = TerrainMeshData::default();
        mesh.authored_cells = vec![None; cells.len()];
        mesh.footing_heights = (0..cells.len()).map(|i| i as f32 * 0.02).collect();
        let footing = mesh.footing_heights.clone();
        assert!(append(&mut mesh, &refs, &g, p, &mut claimed));
        assert_eq!(mesh.footing_heights, footing);
        assert_eq!(claimed.iter().filter(|&&v| v).count(), owned);
        assert_eq!(
            mesh.authored_cells.iter().filter(|v| v.is_some()).count(),
            owned
        );
        assert!(
            mesh.textured.positions[..owned * 4]
                .iter()
                .all(|p| p[1] == 0.)
        );
        assert!(mesh.solid.cutaway_ranges.is_empty() && mesh.textured.cutaway_ranges.is_empty());
        let body=furniture_body(&mesh,owned);
        assert!(mesh.solid.positions.is_empty());
        assert!(!body.positions.is_empty());
        let ([west, east, north, south], rise) = kind.fitting();
        assert!(
            body
                .positions
                .iter()
                .all(|p| p.iter().all(|v| v.is_finite())
                    && p[0] >= g.origin_x + west - 0.001
                    && p[0] <= g.origin_x + east + 0.001
                    && p[1] >= -0.001
                    && p[1] <= rise + 0.001
                    && p[2] >= g.origin_z + north - 0.001
                    && p[2] <= g.origin_z + south + 0.001)
        );
        assert!(
            body
                .normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
        );
        let before = mesh.clone();
        assert!(!append(&mut mesh, &refs, &g, p, &mut claimed));
        assert_eq!(mesh, before);
        if kind == Kind::SideDesk {
            assert!(
                (0..4).all(|x| !claimed[3 * g.width + x]),
                "native lower floor retained"
            );
        }
        if kind == Kind::MeetingTable {
            for y in 0..8 {
                for x in 0..8 {
                    assert_eq!(
                        claimed[y * g.width + x],
                        (2..6).contains(&y) && x >= 2,
                        "chairs and surrounding floor are separate from the table"
                    );
                }
            }
        }
    }
}
#[test]
fn kanto_native_silph_reception_counter_keeps_native_l_and_clerk_aisle() {
    let (mut cells,g,map)=fixture(Kind::ReceptionCounter);
    let empty=vec![false;cells.len()];
    let refs=cells.iter().collect::<Vec<_>>();
    let placements=resolve(map,&refs,&g,[0,0],None,&empty);
    let p=placements.iter().find(|p|p.resolved.kind()==Kind::ReceptionCounter).unwrap();
    let mut mesh=TerrainMeshData{footing_heights:vec![1.25;cells.len()],
        authored_cells:vec![None;cells.len()],..Default::default()};
    let mut claimed=empty.clone();
    assert!(append(&mut mesh,&refs,&g,p,&mut claimed));
    assert_eq!(claimed.iter().filter(|&&v|v).count(),34);
    assert!(!claimed[5*g.width+9],"native receptionist foot must remain separate");
    assert!(mesh.footing_heights.iter().all(|&v|v==1.25));
    assert!(mesh.solid.positions.is_empty());
    let body=&mesh.textured.positions[34*4..];
    assert!(body.iter().all(|v|v[1]>=0. && v[1]<=8.
        && (v[0]>=g.origin_x+80.-0.001 || v[2]>=g.origin_z+48.-0.001)));
    for ids in mesh.textured.indices[34*6..].chunks_exact(3) {
        let [a,b,c]=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
        let normal=Vec3::from_array(mesh.textured.normals[ids[0]as usize]);
        assert!((b-a).cross(c-a).dot(normal)>0.);
    }
    let top=g.uv(p.resolved.column,p.resolved.row+6);
    let lower=g.uv(p.resolved.column,p.resolved.row+7);
    let dark=[lower.0+(lower.1-lower.0)*4.5/8.,lower.2+(lower.3-lower.2)*1.5/8.];
    let light=[top.0+(top.1-top.0)*4.5/8.,top.2+(top.3-top.2)*1.5/8.];
    let body_uv=&mesh.textured.uvs[34*4..];
    assert!(body_uv.contains(&dark),"native dark lower fascia must survive");
    assert!(body_uv.contains(&light),"native light rim must survive");
    for y in 0..8 {for x in 0..12 {
        let i=y*g.width+x;cells[i].source.tile_index^=1;
        assert!(!resolve(map,&cells.iter().collect::<Vec<_>>(),&g,[0,0],None,&empty)
            .iter().any(|p|p.resolved.kind()==Kind::ReceptionCounter));
        cells[i].source.tile_index^=1;
    }}
    assert!(resolve(map,&cells.iter().collect::<Vec<_>>(),&g,[1,0],None,&empty)
        .iter().all(|p|p.resolved.kind()!=Kind::ReceptionCounter));
    let custom:Document=serde_json::from_str(r#"{"objects":[{"name":"Custom clerk floor","map":"SilphCo1F","tileset":"facility","metatile":52,"origin":[0,0],"tiles":[[1,38,1,38]],"ground":1,"top_pixels":0,"depth_pixels":8}]}"#).unwrap();
    assert!(resolve(map,&cells.iter().collect::<Vec<_>>(),&g,[0,0],Some(&custom),&empty)
        .iter().all(|p|p.resolved.kind()!=Kind::ReceptionCounter));
}

#[test]
fn kanto_native_silph_sofa_has_closed_blue_volume_and_separate_native_floor() {
    let (mut cells,g,map)=fixture(Kind::Sofa);
    let empty=vec![false;cells.len()];
    let refs=cells.iter().collect::<Vec<_>>();
    let placements=resolve(map,&refs,&g,[0,0],None,&empty);
    assert_eq!(placements.len(),1);
    let p=&placements[0];
    let mut mesh=TerrainMeshData{footing_heights:vec![1.25;cells.len()],
        authored_cells:vec![None;cells.len()],..Default::default()};
    let mut claimed=empty.clone();
    assert!(append(&mut mesh,&refs,&g,p,&mut claimed));
    assert!(mesh.solid.positions.is_empty());
    assert_eq!(claimed.iter().filter(|&&v|v).count(),8);
    assert_eq!(mesh.textured.cutaway_ranges,vec![8*4..mesh.textured.positions.len()],
        "only the complete sofa body fades; floor remains opaque");
    let (opaque,groups)=crate::occluder_fade::prepare(mesh.textured.clone());
    assert_eq!(groups.len(),1,"all upholstery and feet share one fade owner");
    assert_eq!(opaque.positions.len(),8*4,"native floor remains independent");
    assert!((2..4).all(|y|(0..4).all(|x|!claimed[y*g.width+x])));
    assert!(mesh.footing_heights.iter().all(|&v|v==1.25));
    let body=&mesh.textured.positions[8*4..];
    assert!(body.iter().all(|v|v.iter().all(|n|n.is_finite())
        && v[0]>=g.origin_x+1.-0.001 && v[0]<=g.origin_x+31.+0.001
        && v[2]>=g.origin_z+1.-0.001 && v[2]<=g.origin_z+15.+0.001));
    assert!((body.iter().map(|v|v[1]).fold(0.,f32::max)-11.).abs()<0.001);
    for ids in mesh.textured.indices[8*6..].chunks_exact(3) {
        let [a,b,c]=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
        let normal=Vec3::from_array(mesh.textured.normals[ids[0]as usize]);
        assert!((b-a).cross(c-a).dot(normal)>0.,"closed sofa winding");
    }
    for y in 0..4 {for x in 0..4 {
        let i=y*g.width+x;cells[i].source.tile_index^=1;
        assert!(resolve(map,&cells.iter().collect::<Vec<_>>(),&g,[0,0],None,&empty).is_empty());
        cells[i].source.tile_index^=1;
    }}
    assert!(resolve("PowerPlant",&cells.iter().collect::<Vec<_>>(),&g,[0,0],None,&empty).is_empty());
    let custom:Document=serde_json::from_str(r#"{"objects":[{"name":"Custom sofa floor","map":"SilphCo1F","tileset":"facility","metatile":45,"origin":[0,2],"tiles":[[1,38,1,38]],"ground":1,"top_pixels":0,"depth_pixels":8}]}"#).unwrap();
    assert!(resolve(map,&cells.iter().collect::<Vec<_>>(),&g,[0,0],Some(&custom),&empty).is_empty());
}

#[test]
fn kanto_native_silph_meeting_table_keeps_gold_complete_plot_and_footing() {
    let (mut cells,g,_)=fixture(Kind::MeetingTable);
    let empty=vec![false;cells.len()];
    let refs=cells.iter().collect::<Vec<_>>();
    let placements=resolve("SilphCo1F",&refs,&g,[0,0],None,&empty);
    let p=placements.iter().find(|p|p.resolved.kind()==Kind::MeetingTable).unwrap();
    let mut mesh=TerrainMeshData{footing_heights:vec![2.5;cells.len()],
        authored_cells:vec![None;cells.len()],..Default::default()};
    let mut claimed=empty.clone();
    assert!(append(&mut mesh,&refs,&g,p,&mut claimed));
    assert!(mesh.solid.positions.is_empty(),"generic cream table must be replaced");
    assert_eq!(claimed.iter().filter(|&&v|v).count(),24);
    assert!(mesh.footing_heights.iter().all(|&v|v==2.5));
    let uv=g.uv(3,3);let gold=[uv.0+(uv.1-uv.0)*4.5/8.,uv.2+(uv.3-uv.2)*4.5/8.];
    assert!(mesh.textured.uvs[24*4..].iter().all(|&uv|uv==gold));
    let body=&mesh.textured.positions[24*4..];
    assert!(body.iter().all(|v|v[0]>=g.origin_x+16.-0.001 && v[0]<=g.origin_x+64.+0.001
        && v[2]>=g.origin_z+16.-0.001 && v[2]<=g.origin_z+48.+0.001));
    assert!((body.iter().map(|v|v[1]).fold(0.,f32::max)-9.135).abs()<0.001);
    for y in 0..8 {for x in 0..8 {
        let i=y*g.width+x;cells[i].source.tile_index^=1;
        assert!(!resolve("SilphCo1F",&cells.iter().collect::<Vec<_>>(),&g,[0,0],None,&empty)
            .iter().any(|p|p.resolved.kind()==Kind::MeetingTable));
        cells[i].source.tile_index^=1;
    }}
}

#[test]
fn facility_custom_floor_guard_and_stale_source_fail_without_partial_append() {
    let (mut cells, g, map) = fixture(Kind::SideDesk);
    let empty = vec![false; cells.len()];
    let custom:Document=serde_json::from_str(r#"{"objects":[{"name":"Custom native floor beside desk","map":"RuinsOfAlphResearchCenter","tileset":"facility","metatile":41,"origin":[0,3],"tiles":[[38,1,38,1]],"ground":1,"top_pixels":0,"depth_pixels":8}]}"#).unwrap();
    assert!(
        resolve(
            map,
            &cells.iter().collect::<Vec<_>>(),
            &g,
            [0, 0],
            Some(&custom),
            &empty
        )
        .is_empty()
    );
    let placements = resolve(
        map,
        &cells.iter().collect::<Vec<_>>(),
        &g,
        [0, 0],
        None,
        &empty,
    );
    assert_eq!(placements.len(), 1);
    for i in [
        0,
        3 * g.width,
        placements[0].resolved.ground[0],
        placements[0].resolved.ground[1],
    ] {
        cells[i].source.tile_index ^= 1;
        let mut mesh = TerrainMeshData::default();
        let mut claimed = empty.clone();
        assert!(!append(
            &mut mesh,
            &cells.iter().collect::<Vec<_>>(),
            &g,
            &placements[0],
            &mut claimed
        ));
        assert_eq!(mesh, TerrainMeshData::default());
        assert_eq!(claimed, empty);
        cells[i].source.tile_index ^= 1;
    }
}
#[test]
fn facility_document_face_is_live_outward_and_clears_its_physical_paper() {
    for kind in [Kind::SquareTable, Kind::SideDesk] {
        let (cells, g, map) = fixture(kind);
        let mut claimed = vec![false; cells.len()];
        let refs: Vec<_> = cells.iter().collect();
        let placements = resolve(map, &refs, &g, [0, 0], None, &claimed);
        let p = placements
            .iter()
            .find(|p| p.resolved.kind() == kind)
            .unwrap();
        let mut mesh = TerrainMeshData::default();
        assert!(append(&mut mesh, &refs, &g, p, &mut claimed));
        let count = mesh.textured.positions.len()-4;
        assert!(
            mesh.textured.positions[count..]
                .iter()
                .all(|p| (p[1] - 9.835).abs() < 0.001)
        );
        for tri in mesh.textured.indices.chunks_exact(3) {
            let a = Vec3::from_array(mesh.textured.positions[tri[0] as usize]);
            let b = Vec3::from_array(mesh.textured.positions[tri[1] as usize]);
            let c = Vec3::from_array(mesh.textured.positions[tri[2] as usize]);
            let n = Vec3::from_array(mesh.textured.normals[tri[0] as usize]);
            assert!((b - a).cross(c - a).dot(n) > 0.);
        }
        let source_row = if kind == Kind::SquareTable { 2 } else { 1 };
        let uv = g.uv(p.resolved.column + 1, p.resolved.row + source_row);
        let u = |pixel: f32| uv.0 + (uv.1 - uv.0) * pixel / 8.;
        let v = |pixel: f32| uv.2 + (uv.3 - uv.2) * pixel / 8.;
        assert_eq!(mesh.textured.positions.len() - count, 4);
        assert_eq!(
            &mesh.textured.uvs[count..],
            &[
                [u(1.), v(7.)],
                [u(7.), v(7.)],
                [u(7.), v(1.)],
                [u(1.), v(1.)],
            ],
            "the exact 6x6 native ink face keeps its orientation on the page"
        );
    }
}
