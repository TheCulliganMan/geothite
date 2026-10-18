//! One source-complete north gate, including Johto/modern connections.
use super::*;
const ROOF:[[u16;16];2]=[
    [49,54,54,54,65,72,72,72,65,72,72,72,81,82,82,82],
    [54,54,54,52,72,72,72,68,72,72,72,68,82,82,82,84],
];
const FACADE:[[u16;16];2]=[
    [26,27,27,27,26,27,27,27,26,27,55,56,1,2,57,58],
    [27,27,27,28,27,38,38,28,27,27,27,28,2,2,2,22],
];
pub(super) fn matches(cells:&[&VisualTile],g:&GridGeometry,p:BuildingPlacement)->bool {
    if p.width!=8 || p.height!=8 || p.column+8>g.width || p.row+8>g.height {return false;}
    let Some(facade)=cells.get((p.row+4)*g.width+p.column).map(|c|c.source.tileset_id.as_ref()) else {return false;};
    if !matches!(facade,"johto"|"johto_modern") {return false;}
    (0..8).all(|y|(0..8).all(|x| {
        let Some(tile)=cells.get((p.row+y)*g.width+p.column+x) else {return false;};
        let s=&tile.source;
        let (family,block,art)=if y<4 {("johto",[8,9][x/4],ROOF[x/4])}
            else {(facade,[0x16,0x1e][x/4],FACADE[x/4])};
        s.tileset_id.as_ref()==family && s.metatile_id==block
            && s.subtile_column as usize==x%4 && s.subtile_row as usize==y%4
            && s.tile_index==art[y%4*4+x%4]
    })) && super::super::outdoor_entry_mat::north_approach(cells,g,p)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->(Vec<VisualTile>,GridGeometry,BuildingPlacement) {
        let g=GridGeometry {width:12,height:16,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut tiles=(0..g.width*g.height).map(|i|VisualTile {
            column:(i%g.width)as u32,row:(i/g.width)as u32,texture:Default::default(),priority:false,animation_frames:None,
            source:VisualTileSource {tileset_id:"johto".into(),metatile_id:1,
                subtile_column:(i%4)as u8,subtile_row:(i/g.width%4)as u8,tile_index:6}
        }).collect::<Vec<_>>();
        let p=BuildingPlacement {column:2,row:4,width:8,height:8,roof_rows:4,ground_tile_index:6};
        for y in 0..12 {for x in 0..8 {
            let s=&mut tiles[y*g.width+x+2].source;
            if y<4 {
                s.metatile_id=[0x0c,0x0d][x/4];
                s.tile_index=if y<2 {6}else if x<2 || x>=6 {if y==2 {0x5a}else{0x59}}else{0x40};
            } else if y<8 {s.metatile_id=[8,9][x/4];s.tile_index=ROOF[x/4][y%4*4+x%4];}
            else {s.tileset_id="johto_modern".into();s.metatile_id=[0x16,0x1e][x/4];s.tile_index=FACADE[x/4][y%4*4+x%4];}
            s.subtile_column=(x%4)as u8;s.subtile_row=(y%4)as u8;
        }}
        for y in 12..16 {for x in 0..g.width {tiles[y*g.width+x].source.tileset_id="johto_modern".into();}}
        (tiles,g,p)
    }
    #[test]
    fn mixed_source_gate_uses_one_shared_open_passage_from_either_map() {
        let (tiles,g,p)=fixture();let cells=tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|t|shape_for_source(&t.source)).collect::<Vec<_>>();
        let mut reference=None;
        for map in ["Route35","GoldenrodCity","UnseenTown"] {
            let plots=building_placements(map,&cells,&g);
            assert_eq!(plots.iter().filter(|&&q|q==p).count(),1);
            let d=descriptor(map,&cells,&g,p).unwrap();
            assert_eq!(d.asset,Asset::World(Kind::EastGate));
            assert_eq!(d.north_aperture,Some([2.,6.]));assert!(!d.east);assert_eq!(d.door,Some(4.));
            let mut mesh=TerrainMeshData {footing_heights:vec![0.;cells.len()],authored_cells:vec![None;cells.len()],..Default::default()};
            let mut claimed=vec![false;cells.len()];
            assert!(append_building(&mut mesh,&cells,&shapes,&g,map,p,&mut claimed,[0,0]));
            assert_eq!(claimed.iter().filter(|&&v|v).count(),64);
            assert_eq!(mesh.footing_heights,vec![0.;cells.len()]);
            for y in 4..12 {for x in 2..10 {assert_eq!(mesh.authored_cells[y*g.width+x],Some("world-exterior/east_gate"));}}
            for (positions,uv) in mesh.textured.positions.chunks_exact(4).zip(mesh.textured.uvs.chunks_exact(4)) {
                let sample=super::super::new_bark::sampled_cell(uv,&g).unwrap();
                let x=((positions[0][0]-g.origin_x)/g.tile_width).round()as usize;
                let y=((positions[0][2]-g.origin_z)/g.tile_height).round()as usize;
                assert_eq!(cells[sample].source.tileset_id,cells[y*g.width+x].source.tileset_id,
                    "a mixed gate floor must retain each source family's finish");
                assert!(positions.iter().all(|p|p[1]==0.));
            }
            if let Some(old)=&reference {assert_eq!(&mesh.solid,old,"one asset across the connection");}
            else {reference=Some(mesh.solid.clone());}
            super::super::new_bark::polish_world_surfaces(&mut mesh,map,&cells,&g,[0,0]);
            assert!(mesh.textured.indices.is_empty(),"no rejected mint floor strips under the threshold");
        }
    }
    #[test]
    fn same_family_north_gate_reuses_the_shared_passage_and_native_floor() {
        let (mut tiles,g,p)=fixture();
        for y in 8..16 {for x in 0..g.width {tiles[y*g.width+x].source.tileset_id="johto".into();}}
        let cells=tiles.iter().collect::<Vec<_>>();let shapes=cells.iter().map(|t|shape_for_source(&t.source)).collect::<Vec<_>>();
        let mut reference=None;
        for map in ["RuinsOfAlphOutside","Route36","UnseenTown"] {
            assert!(matches(&cells,&g,p));
            assert_eq!(building_placements(map,&cells,&g).iter().filter(|&&q|q==p).count(),1);
            let d=descriptor(map,&cells,&g,p).unwrap();
            assert_eq!(d.asset,Asset::World(Kind::EastGate));assert_eq!(d.north_aperture,Some([2.,6.]));
            let mut mesh=TerrainMeshData{footing_heights:vec![0.;cells.len()],authored_cells:vec![None;cells.len()],..Default::default()};
            let mut claimed=vec![false;cells.len()];
            assert!(append_building(&mut mesh,&cells,&shapes,&g,map,p,&mut claimed,[0,0]));
            assert_eq!(claimed.iter().filter(|&&c|c).count(),64);
            assert!(mesh.footing_heights.iter().all(|h|*h==0.));
            assert!(mesh.authored_cells.iter().flatten().all(|label|*label=="world-exterior/east_gate"));
            for uv in mesh.textured.uvs.chunks_exact(4) {
                let i=super::super::new_bark::sampled_cell(uv,&g).unwrap();
                assert_eq!(cells[i].source.tileset_id.as_ref(),"johto");
            }
            if let Some(old)=&reference {assert_eq!(&mesh.solid,old);}else{reference=Some(mesh.solid.clone());}
        }
        for y in 0..4 {for x in 2..10 {
            let mut altered=tiles.clone();altered[y*g.width+x].source.tile_index^=1;
            assert!(!matches(&altered.iter().collect::<Vec<_>>(),&g,p),"all approach cells are required");
        }}
    }
    #[test]
    fn mixed_gate_requires_every_roof_facade_and_approach_source_cell() {
        let (tiles,g,p)=fixture();
        assert!(matches(&tiles.iter().collect::<Vec<_>>(),&g,p));
        for y in 0..12 {for x in 2..10 {
            let mut altered=tiles.clone();altered[y*g.width+x].source.tile_index^=1;
            let refs=altered.iter().collect::<Vec<_>>();
            assert!(!matches(&refs,&g,p),"reject changed cell{x},{y}");
            assert!(!building_placements("Route35",&refs,&g).contains(&p));
            assert!(descriptor("Route35",&refs,&g,p).is_none());
        }}
        let mut same_family=tiles.clone();for y in 8..12 {for x in 2..10 {same_family[y*g.width+x].source.tileset_id="johto".into();}}
        // An ordinary house lacks the complete gate approach. Same-family
        // drawing alone cannot change its asset identity.
        for y in 0..4 {for x in 2..10 {same_family[y*g.width+x].source.metatile_id=1;}}
        assert!(!matches(&same_family.iter().collect::<Vec<_>>(),&g,p),"ordinary same-family houses retain their producer");
        assert_eq!(descriptor("MahoganyTown",&same_family.iter().collect::<Vec<_>>(),&g,p).unwrap().asset,Asset::Johto(Johto::PlayerHouse));
    }
}
