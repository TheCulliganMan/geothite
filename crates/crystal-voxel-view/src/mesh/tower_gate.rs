//! The existing passage gate, selected by its complete shifted Tower drawing.
use super::*;
const BLOCKS:[[u16;3];3]=[[0x2d,0x0c,0x0d],[0x2e,0x18,0x2f],[0x30,0x16,0x31]];
const ART:[[[u16;16];3];3]=[
    [[0x1e,0x1f,0,0,0x13,0x15,0,0,0x13,0x15,0x5a,0x5a,0x3e,0x3f,0x59,0x59],
     [0,0,0,0,0,0,6,0,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40],
     [0,0,0x1e,0x1f,0,0,0x13,0x15,0x5a,0x5a,0x13,0x15,0x59,0x59,0x3e,0x3f]],
    [[0x1e,0x1f,0x10,0x11,0x13,0x15,0x0d,0x0e,0x13,0x15,0x0d,0x0e,0x3e,0x3f,0x0a,0x0b],
     [0x11,0x11,0x11,0x11,0x0e,0x0e,0x0e,0x0e,0x0e,0x0e,0x0e,0x0e,0x0b,0x0b,0x0b,0x0b],
     [0x11,0x12,0,0,0x0e,0x0f,0,0,0x0e,0x0f,0x5a,0x5a,0x0b,0x0c,0x59,0x59]],
    [[0x1e,0x1f,0x1a,0x1b,0x13,0x15,0x1a,0x1b,0x13,0x15,0x1a,0x1b,0x3e,0x3f,1,2],
     [0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x1b,0x26,0x37,0x38,0x1b,0x1b,0x39,0x3a,2,2],
     [0x1b,0x1c,6,6,0x26,0x1c,6,6,0x1b,0x1c,6,6,2,0x16,6,6]]
];
pub(in crate::mesh) fn matches(cells:&[&VisualTile],g:&GridGeometry,p:BuildingPlacement)->bool {
    p.width==8 && p.height==8 && p.row>=4 && p.column+8<=g.width && p.row+8<=g.height
        && (0..12).all(|y|(0..8).all(|x| {
            let Some(t)=cells.get((p.row-4+y)*g.width+p.column+x) else {return false;};
            let s=&t.source;let sx=x+2;
            s.tileset_id.as_ref()=="battle_tower_outside" && s.metatile_id==BLOCKS[y/4][sx/4]
                && s.subtile_column as usize==sx%4 && s.subtile_row as usize==y%4
                && s.tile_index==ART[y/4][sx/4][y%4*4+sx%4]
        }))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->(Vec<VisualTile>,GridGeometry,BuildingPlacement) {
        let mut sources=Vec::new();
        for y in 0..12 {for x in 0..12 {
            let mut s=crate::mesh::tests::source_with_tile(BLOCKS[y/4][x/4],(x%4)as u8,(y%4)as u8,ART[y/4][x/4][y%4*4+x%4]);
            s.tileset_id="battle_tower_outside".into();sources.push(s);
        }}
        let f=crate::mesh::tests::frame(12,12,sources);
        let g=GridGeometry{width:12,height:12,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        (f.tiles,g,BuildingPlacement{column:2,row:4,width:8,height:8,roof_rows:4,ground_tile_index:6})
    }
    #[test]
    fn tower_gate_uses_shared_open_passage_and_preserves_neighbor_trees_rails_and_support() {
        let (tiles,g,p)=fixture();let cells=tiles.iter().collect::<Vec<_>>();
        for map in ["BattleTowerOutside","UnseenTown"] {
            assert!(building_placements(map,&cells,&g).contains(&p));
            let d=descriptor(map,&cells,&g,p).unwrap();
            assert_eq!(d.asset,Asset::World(Kind::EastGate));assert_eq!(d.north_aperture,Some([2.,6.]));
            assert!(!d.east);assert_eq!(d.door,Some(4.));
            let shapes=cells.iter().map(|t|shape_for_source(&t.source)).collect::<Vec<_>>();
            let mut mesh=TerrainMeshData{footing_heights:vec![0.;144],authored_cells:vec![None;144],..Default::default()};
            let mut claimed=vec![false;144];
            assert!(append_building(&mut mesh,&cells,&shapes,&g,map,p,&mut claimed,[0,0]));
            assert_eq!(claimed.iter().filter(|v|**v).count(),64);
            assert_eq!(mesh.footing_heights,vec![0.;144]);
            assert!(mesh.solid.positions.iter().any(|v|v[1]>32.));
            for y in 0..12 {for x in 0..12 {assert_eq!(claimed[y*12+x],y>=4&&(2..10).contains(&x));}}
        }
    }
    #[test]
    fn tower_gate_requires_all_96_source_cells_and_rejects_cropped_phases() {
        let (tiles,g,p)=fixture();assert!(matches(&tiles.iter().collect::<Vec<_>>(),&g,p));
        for y in 0..12 {for x in 2..10 {for field in 0..4 {
            let mut altered=tiles.clone();let s=&mut altered[y*12+x].source;
            match field {0=>s.tile_index^=1,1=>s.metatile_id^=1,2=>s.subtile_column^=1,_=>s.tileset_id="johto".into()}
            let refs=altered.iter().collect::<Vec<_>>();assert!(!matches(&refs,&g,p));
            assert!(descriptor("BattleTowerOutside",&refs,&g,p).is_none());
        }}}
        assert!(!matches(&tiles.iter().collect::<Vec<_>>(),&g,BuildingPlacement{width:7,..p}));
    }
}
