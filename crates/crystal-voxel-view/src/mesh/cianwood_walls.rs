// Timber perimeters use vertical architecture. Scope complete side-strip
// drawings by map and source without changing puzzles or support.
fn append_cianwood_walls(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    let sprout = matches!(map, "SproutTower1F" | "SproutTower2F" | "SproutTower3F");
    let tin=super::modeled_dungeons::tin_tower_guardian_map(map);
    if !sprout && !tin && !matches!(map, "CianwoodGym" | "EcruteakGym" | "BurnedTower1F") {
        return;
    }
    let Some(ground) = ground_sample(cells, map, "tower", crate::tower::TOWER_FLOOR_TILE) else {
        return;
    };
    for (i, t) in cells.iter().enumerate() {
        let s = &t.source;
        // These six complete drawings are the observed tower side perimeters.
        // Gym guardian joins and invisible-field drawings keep separate owners.
        if sprout && !matches!(s.metatile_id, 0x04 | 0x06 | 0x08 | 0x0a | 0x0c | 0x0e) {
            continue;
        }
        let (ox, oy, h) = match s.metatile_id {
            0x04 => (0, 2, 2),
            0x06 => (2, 2, 2),
            0x08 => (0, 0, 4),
            0x0a => (2, 0, 4),
            0x0c | 0x16 => (0, 0, 2),
            0x0e | 0x17 => (2, 0, 2),
            0x21 => (2, 0, 2),
            0x22 => (0, 0, 2),
            _ => continue,
        };
        if s.tileset_id.as_ref() != "tower"
            || s.subtile_column as usize != ox
            || s.subtile_row as usize != oy
        {
            continue;
        }
        let (column, row) = (i % g.width, i / g.width);
        if column + 2 > g.width || row + h > g.height {
            continue;
        }
        if !(0..h).all(|y| {
            (0..2).all(|x| {
                let j = (row + y) * g.width + column + x;
                let t = &cells[j].source;
                !claimed[j]
                    && t.tileset_id == s.tileset_id
                    && t.metatile_id == s.metatile_id
                    && t.subtile_column as usize == ox + x
                    && t.subtile_row as usize == oy + y
                    && t.tile_index
                        == if matches!(s.metatile_id, 0x21 | 0x22) {
                            [0x11, 0x21][y]
                        } else {
                            [0x41, 0x31][x]
                        }
            })
        }) {
            continue;
        }
        // Southern returns must reach the folded facade in the lower two
        // source rows. Their drawing ends at row 2, but the facade stands at
        // row 4; using the short drawing depth leaves a full-row corner gap.
        // Extend only across the complete verified wall, never a floor/profile.
        let block_x = column.saturating_sub(ox);
        let return_join = matches!(s.metatile_id, 0x0c | 0x0e | 0x16 | 0x17)
            && column >= ox && block_x + 4 <= g.width && row + 4 <= g.height
            && (2..4).all(|dy| (0..4).all(|dx| {
                let j=(row+dy)*g.width+block_x+dx;
                let source=&cells[j].source;
                !claimed[j] && source.tileset_id==s.tileset_id
                    && source.metatile_id==s.metatile_id
                    && source.subtile_column as usize==dx
                    && source.subtile_row as usize==dy
                    && matches!(crate::tower::tower_shape(source),Some(CellShape::FacadeBand{band_count:2,..}))
            }));
        // A longitudinal strip ends two source rows before the folded
        // transverse face. Carry its closed timber through that wall course.
        let north_join = sprout && h == 4 && row + h + 2 <= g.height
            && (0..2).all(|dy| (0..2).all(|dx| {
                let j = (row + h + dy) * g.width + column + dx;
                !claimed[j] && matches!(crate::tower::tower_shape(&cells[j].source),
                    Some(CellShape::FacadeBand { band_from_top, band_count: 2, .. })
                        if band_from_top as usize == dy)
            }));
        let p = Placement {
            kind: ModelKind::WallTraditional,
            column,
            row,
            width: 2,
            height: h,
            ground,
            depth_pixels: if matches!(s.metatile_id, 0x21 | 0x22) {
                3.0
            } else if north_join {
                (h + 2) as f32 * 8.0
            } else if return_join {
                32.0
            } else {
                h as f32 * 8.0
            },
            height_pixels: 32.0,
            base_pixels: 0.0,
            footing_pixels: None,
        };
        for j in p.indices(g.width) {
            claimed[j] = true;
        }
        out.push(p);
    }
}

#[cfg(test)]
mod cianwood_wall_tests {
    use super::*;
    use std::sync::Arc;
    #[test]
    fn sprout_side_strips_reach_cross_wall_from_both_sides() {
        let g=GridGeometry {width:4,height:8,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        for (block,ox) in [(0x08,0usize),(0x0a,2)] {
            let tiles=(0..32).map(|i| {
                let (x,y)=(i%4,i/4);
                VisualTile {column:x as u32,row:y as u32,texture:Handle::default(),priority:false,animation_frames:None,
                    source:VisualTileSource {tileset_id:"tower".into(),metatile_id:if y<4 {block}else{5},
                        subtile_column:x as u8,subtile_row:(y%4)as u8,
                        tile_index:if y<4 && (ox..ox+2).contains(&x) {[0x41,0x31][x-ox]}
                            else if (4..6).contains(&y) {[0x11,0x21][y-4]}else{2}}}
            }).collect::<Vec<_>>();
            let refs=tiles.iter().collect::<Vec<_>>();let mut out=vec![];let mut claimed=vec![false;32];
            append_cianwood_walls("SproutTower3F",&refs,&g,&mut claimed,&mut out);
            assert_eq!(out.len(),1);assert_eq!(out[0].depth_pixels,48.);
            assert_eq!(claimed.iter().filter(|v|**v).count(),8,"cross wall retains its own source claim");
            let mut mesh=TerrainMeshData {footing_heights:vec![7.;32],..Default::default()};
            assert!(append(&mut mesh,"SproutTower3F",&refs,&g,&out[0],&mut vec![false;32],[0,0]));
            assert_eq!(mesh.footing_heights,vec![7.;32]);
            assert!((mesh.textured.positions.iter().map(|p|p[2]).fold(0.,f32::max)-48.).abs()<0.001);
            for j in (4..6).flat_map(|y|(ox..ox+2).map(move |x|y*4+x)) {
                let mut protected=vec![false;32];protected[j]=true;let mut out=vec![];
                append_cianwood_walls("SproutTower3F",&refs,&g,&mut protected,&mut out);
                assert_eq!(out[0].depth_pixels,32.,"custom cross-wall source keeps authority");
            }
        }
    }
    #[test]
    fn native_guardians_keep_wall_backing_without_stealing_statue_cells() {
        let g=GridGeometry{width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        for block in [0x21,0x22] {
            let ox=if block==0x21 {0}else{2};
            let art=[[0x22,0x23],[0x32,0x33],[0x12,0x13],[0x36,0x37]];
            let tiles:Vec<_>=(0..16).map(|i| {
                let (x,y)=(i%4,i/4);
                VisualTile{column:x as u32,row:y as u32,texture:Handle::default(),
                    priority:false,animation_frames:None,
                    source:VisualTileSource{tileset_id:"tower".into(),metatile_id:block,
                        subtile_column:x as u8,subtile_row:y as u8,
                        tile_index:if (ox..ox+2).contains(&x){art[y][x-ox]}
                            else if y<2 {[0x11,0x21][y]}else{2}}}
            }).collect();
            let refs=tiles.iter().collect::<Vec<_>>();
            for map in (1..=9).map(|level|format!("TinTower{level}F"))
                .chain(["BurnedTower1F".into(),"CianwoodGym".into(),"EcruteakGym".into()]) {
                let mut claimed=vec![false;16];let mut out=vec![];
                append_cianwood_walls(&map,&refs,&g,&mut claimed,&mut out);
                assert_eq!(out.len(),1);
                assert!((0..4).all(|y|(ox..ox+2).all(|x|!claimed[y*4+x])));
                assert_eq!(guardian_wall_bounds(&map,&out[0],&refs,&g,[8.,16.,13.,16.]),[0.,32.,13.,16.]);
                for y in 0..4 { for x in ox..ox+2 { for field in 0..5 {
                    let mut changed=tiles.clone();let source=&mut changed[y*4+x].source;
                    match field {0=>source.tileset_id="other".into(),1=>source.metatile_id^=1,
                        2=>source.subtile_column^=1,3=>source.subtile_row^=1,_=>source.tile_index^=1}
                    assert_eq!(guardian_wall_bounds(&map,&out[0],&changed.iter().collect::<Vec<_>>(),&g,
                        [8.,16.,13.,16.]),[8.,16.,13.,16.],"edited guardian cannot infer backing");
                }}}
                let cropped=GridGeometry{height:3,..g};
                assert_eq!(guardian_wall_bounds(&map,&out[0],&refs[..12],&cropped,
                    [8.,16.,13.,16.]),[8.,16.,13.,16.]);
                assert_eq!(guardian_wall_bounds("UnreviewedGym",&out[0],&refs,&g,
                    [8.,16.,13.,16.]),[8.,16.,13.,16.]);
            }
        }
    }
    #[test]
    fn external_gym_guardian_backing_completes_the_native_north_course() {
        let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack required");
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(std::path::PathBuf::from(path)).unwrap();
        let pack=loaded.pack();let art=&pack.runtime_files()["data/tilesets/tower_metatiles.bin"];
        for map in ["CianwoodGym","EcruteakGym"] {
            let data=&pack.data().maps[map];
            let (w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
            assert_eq!((w,h),(20,36));
            let tiles=(0..w*h).map(|i| {
                let (x,y)=(i%w,i/w);let block=data.blocks[y/4*(w/4)+x/4];
                VisualTile{column:x as u32,row:y as u32,texture:Handle::default(),priority:false,
                    animation_frames:None,source:VisualTileSource{tileset_id:"tower".into(),
                        metatile_id:block,subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,
                        tile_index:art[block as usize*16+y%4*4+x%4]as u16}}
            }).collect::<Vec<_>>();
            let refs=tiles.iter().collect::<Vec<_>>();
            let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let mut mesh=TerrainMeshData{footing_heights:vec![7.;w*h],
                authored_cells:vec![None;w*h],..Default::default()};
            let native=super::super::modeled_dungeons::resolve(map,&refs,&g,None);
            for wall in native.iter().filter(|p|p.native_tower_wall()&&p.source_row()==0) {
                wall.append_room_wall(&mut mesh,&g,&refs,[0,0]);
            }
            let mut reserved=vec![false;w*h];let mut walls=vec![];
            append_cianwood_walls(map,&refs,&g,&mut reserved,&mut walls);
            let mut claimed=vec![false;w*h];let mut guardians=0;
            for wall in walls.iter().filter(|p|p.row==0&&matches!(refs[p.column].source.metatile_id,0x21|0x22)) {
                guardians+=1;
                assert!(append(&mut mesh,map,&refs,&g,wall,&mut claimed,[0,0]));
                let block_x=wall.column-refs[wall.column].source.subtile_column as usize;
                let ox=if refs[wall.column].source.metatile_id==0x21 {0}else{2};
                for y in 0..4 {for x in ox..ox+2 {
                    let i=y*w+block_x+x;
                    assert!(!claimed[i]&&mesh.authored_cells[i].is_none(),"{map}: statue keeps its source");
                }}
            }
            assert_eq!(guardians,2);
            // Upper timber bodies meet over the complete source width;
            // the two statue slots cannot be hidden by floor geometry.
            // Include the beveled crown rather than just its inset top face.
            let mut spans=mesh.textured.indices.chunks_exact(3).filter_map(|tri| {
                let p=[mesh.textured.positions[tri[0]as usize],mesh.textured.positions[tri[1]as usize],
                    mesh.textured.positions[tri[2]as usize]];
                p.iter().all(|p|p[1]>8.&&p[2]>=12.999&&p[2]<=16.001)
                    .then(||(p.iter().map(|p|p[0]).fold(f32::INFINITY,f32::min),
                        p.iter().map(|p|p[0]).fold(f32::NEG_INFINITY,f32::max)))
            }).collect::<Vec<_>>();
            spans.sort_by(|a,b|a.0.total_cmp(&b.0));let mut end=0.;
            for (a,b) in spans {assert!(a<=end+0.001,"{map}: rear wall gap {end}..{a}");end=end.max(b);}
            assert!((end-160.).abs()<0.001,"{map}: full room width has backing");
            assert_eq!(mesh.footing_heights,vec![7.;w*h]);
        }
    }
    #[test]
    fn southern_returns_reach_the_verified_folded_wall_and_reject_overrides() {
        let g=GridGeometry{width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        for block in [0x0c,0x0e,0x16,0x17] {
            let ox=if matches!(block,0x0c|0x16){0}else{2};
            let tiles:Vec<_>=(0..16).map(|i| {
                let x=i%4;let y=i/4;
                VisualTile{column:x as u32,row:y as u32,texture:Handle::default(),
                    priority:false,animation_frames:None,
                    source:VisualTileSource{tileset_id:Arc::from("tower"),metatile_id:block,
                        subtile_column:x as u8,subtile_row:y as u8,
                        tile_index:if y>=2 {if y==2 {0x11}else{0x21}}
                            else if x==ox {0x41}else if x==ox+1 {0x31}else{2}}}
            }).collect();
            let refs=tiles.iter().collect::<Vec<_>>();
            let mut placements=Vec::new();
            append_cianwood_walls("CianwoodGym",&refs,&g,&mut vec![false;16],&mut placements);
            assert_eq!(placements.len(),1);
            assert_eq!(placements[0].depth_pixels,32.);
            assert_eq!(placements[0].indices(4).count(),4,"geometry must not enlarge source claims");
            let mut mesh=TerrainMeshData{footing_heights:vec![3.;16],..Default::default()};
            assert!(append(&mut mesh,"CianwoodGym",&refs,&g,&placements[0],&mut vec![false;16],[0,0]));
            assert_eq!(mesh.footing_heights,vec![3.;16]);
            assert!((mesh.textured.positions.iter().map(|p|p[2]).fold(0.,f32::max)-32.).abs()<0.001);
            for index in 8..16 {
                let mut reserved=vec![false;16];reserved[index]=true;
                let mut changed=Vec::new();
                append_cianwood_walls("CianwoodGym",&refs,&g,&mut reserved,&mut changed);
                assert_eq!(changed.len(),1);
                assert_eq!(changed[0].depth_pixels,16.,"custom neighboring wall must retain ownership");
            }
        }
    }
    #[test]
    fn perimeter_joins_keep_floor_free_in_gyms_and_all_sprout_floors() {
        let g = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        for (block, ox, oy, h) in [
            (4, 0, 2, 2),
            (6, 2, 2, 2),
            (8, 0, 0, 4),
            (10, 2, 0, 4),
            (12, 0, 0, 2),
            (14, 2, 0, 2),
            (22, 0, 0, 2),
            (23, 2, 0, 2),
            (33, 2, 0, 2),
            (34, 0, 0, 2),
        ] {
            let mut t: Vec<VisualTile> = (0..16)
                .map(|i| VisualTile {
                    column: (i % 4) as u32,
                    row: (i / 4) as u32,
                    texture: Handle::default(),
                    priority: false,
                    animation_frames: None,
                    source: VisualTileSource {
                        tileset_id: Arc::from("tower"),
                        metatile_id: 9,
                        subtile_column: (i % 4) as u8,
                        subtile_row: (i / 4) as u8,
                        tile_index: 2,
                    },
                })
                .collect();
            for y in oy..oy + h {
                for x in ox..ox + 2 {
                    t[y * 4 + x].source.metatile_id = block;
                    t[y * 4 + x].source.tile_index = if matches!(block, 0x21 | 0x22) {
                        [0x11, 0x21][y]
                    } else {
                        [0x41, 0x31][x - ox]
                    };
                }
            }
            let mut maps = vec!["CianwoodGym", "EcruteakGym", "BurnedTower1F"];
            maps.extend(["TinTower1F","TinTower2F","TinTower3F","TinTower4F","TinTower5F",
                "TinTower6F","TinTower7F","TinTower8F","TinTower9F"]);
            if matches!(block, 0x04 | 0x06 | 0x08 | 0x0a | 0x0c | 0x0e) {
                maps.extend(["SproutTower1F", "SproutTower2F", "SproutTower3F"]);
            }
            for map in maps {
                let mut p = Vec::new();
                let mut claimed = vec![false; 16];
                append_cianwood_walls(map, &t.iter().collect::<Vec<_>>(), &g, &mut claimed, &mut p);
                assert_eq!(p.len(), 1);
                assert_eq!(claimed.iter().filter(|&&x| x).count(), h * 2);
                let mut mesh = TerrainMeshData {
                    footing_heights: vec![0.0; 16],
                    ..Default::default()
                };
                assert!(append(
                    &mut mesh,
                    map,
                    &t.iter().collect::<Vec<_>>(),
                    &g,
                    &p[0],
                    &mut vec![false; 16],
                    [0, 0]
                ));
                assert_eq!(mesh.footing_heights, vec![0.0; 16]);
                assert!(!mesh.textured.indices.is_empty());
                if t[oy*4+ox].source.tile_index==0x41 {
                    let uv=g.uv(ox,oy);
                    let gold=[uv.0+(uv.1-uv.0)*1.5/8.,uv.2+(uv.3-uv.2)*3.5/8.];
                    let glint=[uv.0+(uv.1-uv.0)*0.5/8.,uv.2+(uv.3-uv.2)*2.5/8.];
                    let object=mesh.textured.positions.iter().zip(&mesh.textured.uvs)
                        .filter(|(p,_)|p[1]>0.).map(|(_,uv)|*uv).collect::<Vec<_>>();
                    assert!(object.contains(&gold),"framing stays on the native gold field");
                    assert!(!object.contains(&glint),"a one-pixel white glint cannot paint the entire frame");
                }
                assert!(mesh.solid.positions.iter().chain(&mesh.textured.positions).all(|p| p[0] >= ox as f32 * 8.0
                    && p[0] <= (ox + 2) as f32 * 8.0
                    && p[2] >= oy as f32 * 8.0
                    && p[2] <= (oy + h) as f32 * 8.0));
                if map.starts_with("SproutTower")||super::super::modeled_dungeons::tin_tower_guardian_map(map)||map=="BurnedTower1F" {
                    for y in oy..oy+h {
                        for x in ox..ox+2 {
                            let i = y*4+x;
                            for field in 0..6 {
                                let mut changed = t.clone();
                                let s = &mut changed[i].source;
                                let mut reserved = vec![false;16];
                                match field {
                                    0 => s.tileset_id = Arc::from("other"),
                                    1 => s.metatile_id ^= 1,
                                    2 => s.subtile_column ^= 1,
                                    3 => s.subtile_row ^= 1,
                                    4 => s.tile_index ^= 1,
                                    _ => reserved[i] = true,
                                }
                                let mut changed_out = Vec::new();
                                append_cianwood_walls(map, &changed.iter().collect::<Vec<_>>(),
                                    &g, &mut reserved, &mut changed_out);
                                assert!(changed_out.is_empty());
                            }
                        }
                    }
                }
                let original = t[oy * 4 + ox].source.tile_index;
                t[oy * 4 + ox].source.tile_index = 0xff;
                let mut bad = Vec::new();
                append_cianwood_walls(
                    map,
                    &t.iter().collect::<Vec<_>>(),
                    &g,
                    &mut vec![false; 16],
                    &mut bad,
                );
                assert!(bad.is_empty());
                t[oy * 4 + ox].source.tile_index = original;
                append_cianwood_walls(
                    "TinTowerRoof",
                    &t.iter().collect::<Vec<_>>(),
                    &g,
                    &mut vec![false; 16],
                    &mut bad,
                );
                assert!(bad.is_empty());
                let mut reserved = vec![false; 16];
                reserved[oy * 4 + ox] = true;
                append_cianwood_walls(
                    map,
                    &t.iter().collect::<Vec<_>>(),
                    &g,
                    &mut reserved,
                    &mut bad,
                );
                assert!(bad.is_empty());
            }
        }
    }
}

// Guardian art occupies the foreground of these wall blocks. Its source
// ownership stays with the statue; the timber course continues behind it.
fn guardian_wall_bounds(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry,
    bounds:[f32;4])->[f32;4] {
    let s=&cells[p.row*g.width+p.column].source;
    if !(super::modeled_dungeons::tin_tower_guardian_map(map)
        ||matches!(map,"BurnedTower1F"|"CianwoodGym"|"EcruteakGym"))
        || !matches!(s.metatile_id,0x21|0x22) {return bounds;}
    let Some(block_x)=p.column.checked_sub(s.subtile_column as usize) else{return bounds;};
    let ox=if s.metatile_id==0x21 {0}else{2};
    let art=[[0x22,0x23],[0x32,0x33],[0x12,0x13],[0x36,0x37]];
    if block_x+4>g.width || p.row+4>g.height || !(0..4).all(|y|(0..2).all(|x|{
        let t=&cells[(p.row+y)*g.width+block_x+ox+x].source;
        t.tileset_id==s.tileset_id && t.metatile_id==s.metatile_id
            && t.subtile_column as usize==ox+x && t.subtile_row as usize==y
            && t.tile_index==art[y][x]
    })) {return bounds;}
    [g.origin_x+block_x as f32*g.tile_width,
        g.origin_x+(block_x+4)as f32*g.tile_width,bounds[2],bounds[3]]
}

fn append_side_wall(
    mesh: &mut SurfaceMeshData,
    bounds: [f32; 4],
    rise: f32,
    facing: f32,
    tile_width: f32,
    source_uv: impl Fn([usize; 2]) -> [f32; 2],
) {
    let [west, east, north, south] = bounds;
    let depth = south - north;
    let thickness = tile_width * 0.75;
    let cx = if facing > 0.0 {
        west + tile_width * 0.5
    } else {
        east - tile_width * 0.5
    };
    let cz = (north + south) * 0.5;
    let first = mesh.positions.len();
    crate::dungeon_models::model(crate::dungeon_models::Kind::TimberWall).append_tower_timber(
        mesh,
        [-depth * 0.5, depth * 0.5, -thickness * 0.5, thickness * 0.5],
        rise,
        source_uv,
    );
    for p in &mut mesh.positions[first..] {
        let [x, y, z] = *p;
        *p = [cx + facing * z, y, cz - facing * x];
    }
    for (n, c) in mesh.normals[first..]
        .iter_mut()
        .zip(&mut mesh.colors[first..])
    {
        let [x, y, z] = *n;
        let old = crate::interior_models::authored_face_color([1.0; 4], Vec3::from_array(*n))[0];
        *n = [facing * z, y, -facing * x];
        let new = crate::interior_models::authored_face_color([1.0; 4], Vec3::from_array(*n))[0];
        for channel in &mut c[..3] {
            *channel *= new / old;
        }
    }
}
