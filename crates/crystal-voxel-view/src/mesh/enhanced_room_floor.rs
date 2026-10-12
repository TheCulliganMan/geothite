// Source-scoped room finishes. Never infer floors from collision: tower voids,
// Ecruteak's invisible path, Violet's pits and every warp retain their source.
fn enhanced_room_floor_map(map: &str) -> bool {
    matches!(
        map,
        "SproutTower1F"
            | "SproutTower2F"
            | "SproutTower3F"
            | "CianwoodGym"
            | "EcruteakGym"
            | "PewterGym"
            | "VioletGym"
            | "CeruleanGym"
            | "MahoganyGym"
            | "BlackthornGym1F"
            | "BlackthornGym2F"
            | "OlivineGym"
            | "FuchsiaGym"
            | "GoldenrodGameCorner"
            | "CeladonGameCorner"
            | "CeladonGameCornerPrizeRoom"
    )
}

fn enhanced_room_floor_source(map: &str, s: &VisualTileSource) -> Option<InteriorFloor> {
    if s.subtile_column >= 4 || s.subtile_row >= 4 {
        return None;
    }
    let phase = s.subtile_row * 4 + s.subtile_column;
    match (map, s.tileset_id.as_ref()) {
        ("FuchsiaGym","lab")=>crate::fuchsia_gym::floor_source(s)
            .then_some(InteriorFloor::FuchsiaNative),
        ("OlivineGym","champions_room") => {
            if s.metatile_id==0x1f && s.tile_index==0x2d {return Some(InteriorFloor::OlivineRunner);}
            let mask:u16=match (s.tile_index,s.metatile_id) {
                (0x53,0x10)=>0xffff,
                (0x53,0x11)=>0xcc00,(0x53,0x13)=>0x3300,
                (0x53,0x16|0x23)=>0x00ff,
                (0x53,0x18|0x1a|0x24)=>0xcccc,
                (0x53,0x19|0x1b|0x27)=>0x3333,
                (0x51,0x1c)=>0xcc00,(0x51,0x1d)=>0xff00,
                (0x51,0x1e)=>0x3300,(0x51,0x20)=>0x00cc,
                (0x51,0x21)=>0x00ff,(0x51,0x22)=>0x0033,
                _=>0,
            };
            (mask & (1<<phase)!=0).then_some(InteriorFloor::OlivineTile)
        }
        ("BlackthornGym1F"|"BlackthornGym2F", "elite_four_room") => {
            let mask:u16=if map=="BlackthornGym1F" {
                match s.metatile_id {
                    0x18|0x1c => 0x00ff,
                    0x1b|0x3a => 0x00cc,
                    0x1d|0x20|0x3b => 0xcccc,
                    0x1e => 0x3333,
                    0x29 => 0x33ff,
                    0x2d => 0xffff,
                    0x30 => 0xffcc,
                    _ => 0,
                }
            }else{
                match s.metatile_id {
                    0x09 => 0xccff,
                    0x10 => 0xcccc,
                    0x1a => 0x3333,
                    0x23 => 0xff00,
                    0x28 => 0x33ff,
                    0x2d => 0xffff,
                    _ => 0,
                }
            };
            (s.tile_index==0x01 && mask & (1<<phase)!=0).then_some(InteriorFloor::BlackthornTile)
        }
        ("MahoganyGym", "elite_four_room") => {
            let ice_mask: u16 = match s.metatile_id {
                0x2e => 0x33ff,
                0x2f => 0xccff,
                0x31 => 0xffcc,
                0x32 => 0xff33,
                0x35 => 0x3333,
                0x36 => 0xcccc,
                0x39 => 0xffff,
                _ => 0,
            };
            if ice_mask & (1 << phase) != 0 {
                let tile = [[0x05, 0x06], [0x15, 0x16]][s.subtile_row as usize % 2][s.subtile_column as usize % 2];
                return (s.tile_index == tile).then_some(InteriorFloor::MahoganyIce);
            }
            let grip_mask: u16 = match s.metatile_id {
                0x2e => 0xcc00,
                0x2f => 0x3300,
                0x31 => 0x0033,
                0x32 => 0x00cc,
                _ => 0,
            };
            if grip_mask & (1 << phase) != 0 {
                return (s.tile_index == 0x28).then_some(InteriorFloor::MahoganyGrip);
            }
            let floor_mask: u16 = match s.metatile_id {
                0x1c => 0x00ff,
                0x1d => 0xcccc,
                0x1e => 0x3333,
                0x2d => 0xffff,
                _ => 0,
            };
            (s.tile_index == 0x01 && floor_mask & (1 << phase) != 0).then_some(InteriorFloor::Stone)
        }
        ("CeruleanGym", "port") =>
            crate::port::cerulean_floor_source(map, s).then_some(InteriorFloor::PoolCeramic),
        (
            "SproutTower1F" | "SproutTower2F" | "SproutTower3F" | "CianwoodGym" | "EcruteakGym"
            | "PewterGym",
            "tower",
        ) => {
            // Verified depth fields retain native gold grating at the exact
            // top datum, with a dark opening below. They never become ordinary
            // walkable room floor or an inferred wall.
            let depth_tile = match s.metatile_id {
                0x0b => Some(if s.subtile_column == 0 { 0x1a } else { 0x1b }),
                0x07 => Some(match (s.subtile_column, s.subtile_row) {
                    (0, 0) => 0x0a,
                    (_, 0) => 0x0b,
                    (0, _) => 0x1a,
                    _ => 0x1b,
                }),
                0x0f => Some(if s.subtile_row == 0 { 0x0b } else { 0x1b }),
                _ => None,
            };
            if depth_tile == Some(s.tile_index) {
                return Some(InteriorFloor::TowerDepth);
            }
            let mask: u16 = match s.metatile_id {
                0x04 | 0x21 | 0x33 => 0xcc00,
                0x05 | 0x32 => 0xff00,
                0x06 | 0x22 | 0x34 => 0x3300,
                0x08 | 0x12 | 0x19 | 0x30 | 0x35 => 0xcccc,
                0x09 => 0xffff,
                0x0a | 0x13 | 0x1a | 0x31 | 0x36 => 0x3333,
                // Pillar halves have opposite neighboring floor strips.
                0x10 => 0x3333,
                0x11 => 0xcccc,
                0x0c | 0x16 => 0x00cc,
                0x0d | 0x26 => 0x00ff,
                0x0e | 0x17 => 0x0033,
                0x14 => 0xff33,
                0x15 => 0xffcc,
                0x1d => 0x33ff,
                0x1e => 0xccff,
                0x1f | 0x20 => 0xffcc,
                _ => return None,
            };
            (s.tile_index == 0x02 && mask & (1 << phase) != 0).then_some(InteriorFloor::AmberTimber)
        }
        ("VioletGym", "elite_four_room") => {
            let mask: u16 = match s.metatile_id {
                0x15 => 0xf000,
                0x19 | 0x1c => 0x00ff,
                0x1d => 0xcccc,
                0x1e => 0x3333,
                0x2d => 0xffff,
                0x33 => 0xff00,
                _ => return None,
            };
            (s.tile_index == 0x01 && mask & (1 << phase) != 0).then_some(InteriorFloor::VioletBrick)
        }
        ("GoldenrodGameCorner" | "CeladonGameCorner" | "CeladonGameCornerPrizeRoom", "game_corner") =>
            crate::casino::is_floor_cell(s).then_some(InteriorFloor::ArcadeChecker),
        _ => None,
    }
}

#[cfg(test)]
mod enhanced_room_floor_tests {
    use super::*;
    use std::sync::Arc;
    fn source(set: &str, block: u16, x: u8, y: u8, tile: u16) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from(set),
            metatile_id: block,
            subtile_column: x,
            subtile_row: y,
            tile_index: tile,
        }
    }
    #[test]
    fn olivine_floor_excludes_rocks_stage_edges_stairs_and_entry_carpet() {
        let set="champions_room";
        assert_eq!(interior_floor("OlivineGym",&source(set,0x10,0,0,0x53)),Some(InteriorFloor::OlivineTile));
        assert_eq!(interior_floor("OlivineGym",&source(set,0x1f,0,0,0x2d)),Some(InteriorFloor::OlivineRunner));
        for s in [source(set,0x11,0,0,0x46),source(set,0x1c,0,0,0x53),
            source(set,0x1d,0,1,0x51),source(set,0x21,0,2,0x54),
            source(set,0x23,0,2,0x04),source(set,0x25,2,2,0x53),
            source("custom",0x10,0,0,0x53)] {
            assert_eq!(enhanced_room_floor_source("OlivineGym",&s),None);
        }
        assert_eq!(enhanced_room_floor_source("LancesRoom",&source(set,0x10,0,0,0x53)),None);
        let mut runner=SurfaceMeshData::default();
        native_beveled_floor_cell(&mut runner,[0.,8.,0.,8.],0.,[0,0],(0.,1.,0.,1.));
        let area:f32=runner.indices.chunks_exact(3).map(|ids| {
            let p=[0,1,2].map(|i|Vec3::from_array(runner.positions[ids[i] as usize]));
            (p[1]-p[0]).cross(p[2]-p[0]).y*0.5
        }).sum();
        assert!((area-64.).abs()<0.001,"runner must cover its plot once");
        assert!(runner.positions.iter().all(|p|p[1]<=0. && p[1]>=-0.0481));
        assert!(runner.positions.iter().any(|p|p[1]<0.));
        for (p,uv) in runner.positions.iter().zip(&runner.uvs) {
            assert!((uv[0]-p[0]/8.).abs()<0.0001);
            assert!((uv[1]-p[2]/8.).abs()<0.0001);
        }
    }

    #[test]
    fn olivine_native_floor_keeps_source_colors_and_rejects_redirected_or_raised_samples() {
        let g=GridGeometry {width:4,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let tiles=[source("champions_room",0x10,0,0,0x53),
            source("champions_room",0x1f,1,0,0x2d),
            source("champions_room",0x1d,0,2,0x51),
            source("champions_room",0x23,0,2,0x04)].into_iter().enumerate().map(|(i,source)|
            VisualTile {column:i as u32,row:0,source,texture:Default::default(),
                priority:false,animation_frames:None}).collect::<Vec<_>>();
        let refs=tiles.iter().collect::<Vec<_>>();
        let make=|height:f32,redirect:bool| {
            let mut mesh=TerrainMeshData {footing_heights:vec![7.;4],..Default::default()};
            for x in 0..4 {append_top(&mut mesh.textured,g.bounds(x,0).into(),height,
                g.uv(if redirect && x==0 {1} else {x},0));}
            mesh
        };
        let mut native=make(0.,false);
        assert_eq!(finish_surfaces_excluding(&mut native,"OlivineGym",&refs,&g,[0,0],&[false;4]),3);
        assert_eq!(native.textured.indices.len(),246);
        assert!(native.solid.indices.is_empty());
        assert_eq!(native.footing_heights,vec![7.;4]);
        for tri in native.textured.indices.chunks_exact(3) {for &i in tri {
            let p=native.textured.positions[i as usize];let uv=native.textured.uvs[i as usize];
            assert!(p[1]<=0. && p[1]>=-0.0481);
            assert!((uv[0]-p[0]/32.).abs()<0.0001);
            assert!((uv[1]-p[2]/8.).abs()<0.0001);
        }}
        let mut custom=make(0.,false);
        assert_eq!(finish_surfaces_excluding(&mut custom,"OlivineGym",&refs,&g,[0,0],&[true,false,false,false]),2);
        let mut redirected=make(0.,true);
        assert_eq!(finish_surfaces_excluding(&mut redirected,"OlivineGym",&refs,&g,[0,0],&[false;4]),2);
        let mut raised=make(4.,false);
        assert_eq!(finish_surfaces_excluding(&mut raised,"OlivineGym",&refs,&g,[0,0],&[false;4]),0);
        assert_eq!(raised.textured.indices.len(),24);
    }
    #[test]
    fn mahogany_finish_keeps_override_and_entry_carpet_and_actor_footing() {
        let g = GridGeometry { width:4, height:1, tile_width:8., tile_height:8., origin_x:0., origin_z:0. };
        let sources = [source("elite_four_room",0x39,0,0,0x05),
            source("elite_four_room",0x39,1,0,0x06),
            source("elite_four_room",0x2e,2,2,0x28),
            source("elite_four_room",0x1c,0,2,0x04)];
        let tiles: Vec<_> = sources.into_iter().enumerate().map(|(i,source)|VisualTile {
            column:i as u32, row:0, source, texture:Default::default(), priority:false, animation_frames:None,
        }).collect();
        let refs: Vec<_> = tiles.iter().collect();
        let mut mesh = TerrainMeshData { footing_heights:vec![2.;4], ..Default::default() };
        for x in 0..4 {
            append_top(&mut mesh.textured,g.bounds(x,0).into(),0.,g.uv(x,0));
        }
        assert_eq!(finish_surfaces_excluding(&mut mesh,"MahoganyGym",&refs,&g,[0,0],&[true,false,false,false]),2);
        assert_eq!(mesh.textured.indices.len(),156,"custom ice, warp carpet, live ice and rough native stop patch remain textured");
        assert!(mesh.solid.indices.is_empty());
        assert_eq!(mesh.footing_heights,vec![2.;4]);
        assert!(mesh.solid.positions.iter().all(|p|p[1]==0.));
    }

    #[test]
    fn mahogany_floor_preserves_slide_stop_and_warp_identities() {
        let ice = source("elite_four_room", 0x39, 0, 0, 0x05);
        let stop = source("elite_four_room", 0x2e, 2, 2, 0x28);
        assert_eq!(interior_floor("MahoganyGym", &ice), Some(InteriorFloor::MahoganyIce));
        assert_eq!(interior_floor("MahoganyGym", &stop), Some(InteriorFloor::MahoganyGrip));
        for map in ["VioletGym", "BlackthornGym1F", "IcePath1F"] {
            assert_eq!(enhanced_room_floor_source(map, &ice), None);
        }
        for s in [source("elite_four_room",0x1c,0,2,0x04),
            source("elite_four_room",0x35,2,0,0x2a),
            source("elite_four_room",0x39,1,0,0x05),
            source("elite_four_room",0x2e,2,2,0x05),
            source("custom",0x39,0,0,0x05)] {
            assert_eq!(enhanced_room_floor_source("MahoganyGym", &s), None);
        }
        for scale in [0.5,1.,1.5] {
            let mut grip=SurfaceMeshData::default();
            native_rough_floor_cell(&mut grip,[0.,8.*scale,0.,8.*scale],2.,(0.,1.,0.,1.));
            assert_eq!(grip.indices.len(),96);
            let mut area=0.;
            for tri in grip.indices.chunks_exact(3) {
                let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(grip.positions[i as usize]));
                area+=(points[1]-points[0]).cross(points[2]-points[0]).y/2.;
            }
            assert!((area-64.*scale*scale).abs()<0.001);
            assert!(grip.positions.iter().any(|p|p[1]<2.));
            for (point,uv) in grip.positions.iter().zip(&grip.uvs) {
                assert!(point[1]<=2. && point[1]>=2.-0.048*scale-0.0001);
                assert!((uv[0]-point[0]/(8.*scale)).abs()<0.0001);
                assert!((uv[1]-point[2]/(8.*scale)).abs()<0.0001);
            }
            let mut left = SurfaceMeshData::default();
            let mut right = SurfaceMeshData::default();
            native_beveled_floor_cell(&mut left,[0.,8.*scale,0.,8.*scale],2.,[0,0],(0.,0.5,0.,0.5));
            native_beveled_floor_cell(&mut right,[8.*scale,16.*scale,0.,8.*scale],2.,[1,0],(0.5,1.,0.,0.5));
            for (i,p) in left.positions.iter().enumerate().filter(|(_,p)|p[0]==8.*scale) {
                let j = right.positions.iter().position(|q|q==p).unwrap();
                assert_eq!(left.uvs[i],right.uvs[j],"source motif must meet across quadrants");
            }
            for mesh in [&left,&right] {
                let area: f32 = mesh.indices.chunks_exact(3).map(|ids| {
                    let p=[0,1,2].map(|i|mesh.positions[ids[i] as usize]);
                    let a=Vec3::new(p[1][0]-p[0][0],0.,p[1][2]-p[0][2]);
                    let b=Vec3::new(p[2][0]-p[0][0],0.,p[2][2]-p[0][2]);
                    a.cross(b).length()*0.5
                }).sum();
                assert!((area-64.*scale*scale).abs()<0.001,"ice covers its plot once");
                assert!(mesh.positions.iter().all(|p|p[1]<=2. && p[1]>=2.-0.048*scale-0.0001));
                assert!(mesh.positions.iter().any(|p|p[1]==2.));
                assert!(mesh.normals.iter().all(|n|n[1]>0. && (Vec3::from_array(*n).length()-1.).abs()<0.0001));
                for (p,uv) in mesh.positions.iter().zip(&mesh.uvs) {
                    assert!((uv[0]-p[0]/(16.*scale)).abs()<0.0001);
                    assert!((uv[1]-p[2]/(16.*scale)).abs()<0.0001);
                }
            }
        }
    }

    #[test]
    fn cerulean_perimeter_preserves_footing_and_excludes_bridges_water_and_warps() {
        let gray = source("port", 0x31, 0, 0, 0x0a);
        assert_eq!(enhanced_room_floor_source("CeruleanGym", &gray), Some(InteriorFloor::PoolCeramic));
        assert_eq!(interior_floor("CeruleanGym", &gray), Some(InteriorFloor::PoolCeramic));
        let shape = shape_for_source_on_map("CeruleanGym", &gray);
        assert_eq!(shape, CellShape::Flat);
        assert_eq!(shape.surface_height(8.), crate::port::port_shape(&gray).unwrap().surface_height(8.));
        for s in [
            source("port", 0x26, 0, 0, 0x05),
            source("port", 0x2e, 2, 0, 0x14),
            source("port", 0x33, 0, 2, 0x0c),
            source("port", 0x2b, 0, 0, 0x0f),
            source("port", 0x2e, 2, 0, 0x0a),
            source("custom", 0x31, 0, 0, 0x0a),
        ] {
            assert_eq!(enhanced_room_floor_source("CeruleanGym", &s), None);
        }
        assert_eq!(enhanced_room_floor_source("OlivinePort", &gray), None);
    }
    #[test]
    fn floors_preserve_puzzle_voids_pillars_warps_and_source_overrides() {
        for map in [
            "SproutTower1F",
            "SproutTower2F",
            "SproutTower3F",
            "CianwoodGym",
            "EcruteakGym",
            "PewterGym",
        ] {
            assert_eq!(
                enhanced_room_floor_source(map, &source("tower", 9, 0, 0, 2)),
                Some(InteriorFloor::AmberTimber)
            );
            for b in [1, 7, 0x0b, 0x21, 0x22, 0x37, 0x38, 0x39] {
                assert_eq!(
                    enhanced_room_floor_source(map, &source("tower", b, 0, 0, 2)),
                    None
                );
            }
            assert_eq!(
                enhanced_room_floor_source(map, &source("tower", 0x10, 2, 0, 2)),
                None
            );
            assert_eq!(
                enhanced_room_floor_source(map, &source("tower", 0x10, 0, 0, 2)),
                Some(InteriorFloor::AmberTimber)
            );
            assert_eq!(
                enhanced_room_floor_source(map, &source("tower", 9, 4, 0, 2)),
                None
            );
            assert_eq!(
                enhanced_room_floor_source(map, &source("tower", 9, 0, 0, 3)),
                None
            );
        }
        assert_eq!(
            enhanced_room_floor_source("TinTower1F", &source("tower", 9, 0, 0, 2)),
            None
        );
        assert_eq!(
            enhanced_room_floor_source("VioletGym", &source("elite_four_room", 0x17, 0, 0, 1)),
            None
        );
        assert_eq!(
            enhanced_room_floor_source("VioletGym", &source("elite_four_room", 0x19, 0, 2, 1)),
            None
        );
    }
    #[test]
    fn violet_blue_brick_requires_the_original_floor_identity() {
        let floor = source("elite_four_room", 0x2d, 1, 2, 0x01);
        assert_eq!(enhanced_room_floor_source("VioletGym", &floor), Some(InteriorFloor::VioletBrick));
        for map in ["SaffronGym", "GoldenrodGym", "VioletCity"] {
            assert_eq!(enhanced_room_floor_source(map, &floor), None);
        }
        let mut changed = floor;
        changed.tile_index = 0x02;
        assert_eq!(enhanced_room_floor_source("VioletGym", &changed), None);
    }
    #[test]
    fn tower_depth_finish_keeps_exact_plane_identity_and_scope() {
        for map in ["SproutTower1F", "SproutTower2F", "SproutTower3F"] {
            let s = source("tower", 0x0b, 1, 2, 0x1b);
            assert_eq!(
                enhanced_room_floor_source(map, &s),
                Some(InteriorFloor::TowerDepth)
            );
            assert_eq!(interior_floor(map, &s), Some(InteriorFloor::TowerDepth));
            assert_eq!(
                shape_for_source_on_map(map, &s),
                CellShape::PlaneAt { height: 0. }
            );
            assert_eq!(
                enhanced_room_floor_source(map, &source("tower", 1, 1, 2, 0x1b)),
                None
            );
            assert_eq!(
                enhanced_room_floor_source(map, &source("tower", 0x0b, 0, 2, 0x1b)),
                None
            );
        }
    }
    #[test]
    fn arcade_checker_accepts_every_native_phase_but_no_counter_or_stool() {
        for map in ["GoldenrodGameCorner", "CeladonGameCorner"] {
            for y in 0..4 {
                for x in 0..4 {
                    let tile = [[1, 0x10, 1, 0x10], [0x11, 0x12, 0x11, 0x12]][usize::from(y % 2)]
                        [usize::from(x)];
                    assert_eq!(
                        enhanced_room_floor_source(map, &source("game_corner", 1, x, y, tile)),
                        Some(InteriorFloor::ArcadeChecker)
                    );
                    assert_eq!(
                        enhanced_room_floor_source(map, &source("game_corner", 1, x, y, tile + 1)),
                        None
                    );
                }
            }
            for block in [6, 7, 0x0b] {
                assert_eq!(
                    enhanced_room_floor_source(map, &source("game_corner", block, 0, 0, 1)),
                    None
                );
            }
            assert_eq!(
                enhanced_room_floor_source(map, &source("game_corner", 5, 2, 0, 1)),
                None
            );
            assert_eq!(
                enhanced_room_floor_source(map, &source("game_corner", 8, 0, 2, 1)),
                None
            );
            assert_eq!(
                enhanced_room_floor_source(map, &source("game_corner", 0x0c, 0, 2, 1)),
                None
            );
            assert_eq!(
                enhanced_room_floor_source(map, &source("game_corner", 9, 0, 2, 1)),
                None
            );
        }
    }
    #[test]
    fn arcade_native_floor_preserves_palette_uvs_custom_ownership_and_footing() {
        let g=GridGeometry {width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let tiles=(0..16).map(|i| {
            let x=(i%4)as u8;let y=(i/4)as u8;
            let tile=[1,0x10,1,0x10,0x11,0x12,0x11,0x12][(y%2*4+x)as usize];
            VisualTile {column:x as u32,row:y as u32,source:source("game_corner",1,x,y,tile),
                texture:Default::default(),priority:false,animation_frames:None}
        }).collect::<Vec<_>>();
        let refs=tiles.iter().collect::<Vec<_>>();
        for map in ["GoldenrodGameCorner","CeladonGameCorner","CeladonGameCornerPrizeRoom"] {
            let make=|height:f32,redirect:bool| {
                let mut mesh=TerrainMeshData {footing_heights:vec![5.;16],authored_cells:vec![None;16],..Default::default()};
                for i in 0..16 {append_top(&mut mesh.textured,g.bounds(i%4,i/4).into(),height,
                    if redirect && i==0 {g.uv(3,3)} else {g.uv(i%4,i/4)});}
                mesh
            };
            let mut native=make(0.,false);let mut custom=vec![false;16];custom[5]=true;
            assert_eq!(finish_surfaces_excluding(&mut native,map,&refs,&g,[0,0],&custom),15);
            assert_eq!(native.textured.indices.len(),15*48+6);
            assert!(native.solid.indices.is_empty());
            assert_eq!(native.footing_heights,vec![5.;16]);
            assert_eq!(native.authored_cells,vec![None;16]);
            for tri in native.textured.indices.chunks_exact(3) {for &i in tri {
                let p=native.textured.positions[i as usize];let uv=native.textured.uvs[i as usize];
                assert!((uv[0]-p[0]/32.).abs()<0.0001);
                assert!((uv[1]-p[2]/32.).abs()<0.0001);
                assert!(p[1]<=0. && p[1]>=-0.0481);
            }}
            let mut raised=make(3.,false);
            assert_eq!(finish_surfaces_excluding(&mut raised,map,&refs,&g,[0,0],&[false;16]),0);
            let mut redirected=make(0.,true);
            assert_eq!(finish_surfaces_excluding(&mut redirected,map,&refs,&g,[0,0],&[false;16]),15);
        }
    }
    #[test]
    fn finish_preserves_support_and_custom_floor_and_earns_no_object_claims() {
        let g = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let tiles: Vec<_> = (0..16)
            .map(|i| VisualTile {
                column: (i % 4) as u32,
                row: (i / 4) as u32,
                texture: Default::default(),
                priority: false,
                animation_frames: None,
                source: source("tower", 9, (i % 4) as u8, (i / 4) as u8, 2),
            })
            .collect();
        let cells: Vec<_> = tiles.iter().collect();
        let mut mesh = TerrainMeshData::default();
        mesh.footing_heights = vec![3.5; 16];
        mesh.authored_cells = vec![None; 16];
        for i in 0..16 {
            append_top(
                &mut mesh.textured,
                g.bounds(i % 4, i / 4).into(),
                3.5,
                g.uv(i % 4, i / 4),
            );
        }
        let mut excluded = vec![false; 16];
        excluded[5] = true;
        assert_eq!(
            finish_surfaces_excluding(&mut mesh, "SproutTower1F", &cells, &g, [0, 0], &excluded),
            15
        );
        assert_eq!(mesh.textured.indices.len(), 6);
        assert!(mesh.solid.positions.iter().all(|p| p[1] == 3.5));
        assert_eq!(mesh.footing_heights, vec![3.5; 16]);
        assert_eq!(mesh.authored_cells, vec![None; 16]);
    }
}
