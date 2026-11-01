//! A shallow water funnel on the exact authored Dragon's Den whirlpool slots.
use super::*;
use bevy::math::Vec2;
use std::collections::HashSet;

pub(super) fn append(mesh: &mut TerrainMeshData, cells: &[&VisualTile], g: &GridGeometry) -> HashSet<usize> {
    let selected=crate::cave_water::den_whirlpool_cells(cells,g.width,g.height);
    let base=crate::profile::WATER_HEIGHT*g.tile_height/SOURCE_TILE_HEIGHT;
    let mut claimed=HashSet::new();
    for (i,_) in selected {
        let (col,row)=(i%g.width,i/g.width);
        let s=&cells[i].source;
        let centre=Vec3::new(g.origin_x+(col-s.subtile_column as usize+1) as f32*g.tile_width,
            base,g.origin_z+(row-s.subtile_row as usize+1) as f32*g.tile_height);
        let (w,e,n,z)=g.bounds(col,row);
        let (u0,u1,v0,v1)=g.uv(col,row);
        let at=|x:f32,y:f32| {
            let p=Vec3::new(w+(e-w)*x,base,n+(z-n)*y);
            let r=Vec2::new((p.x-centre.x)/g.tile_width,(p.z-centre.z)/g.tile_height).length();
            let t=((r-0.12)/0.88).clamp(0.,1.);
            p-Vec3::Y*(1.-t*t*(3.-2.*t))*g.tile_height*0.20
        };
        for y in 0..4 {for x in 0..4 {
            let (a,b,c,d)=(x as f32/4.,(x+1) as f32/4.,y as f32/4.,(y+1) as f32/4.);
            let p=[at(a,c),at(a,d),at(b,d),at(b,c)];
            let normal=(p[1]-p[0]).cross(p[3]-p[0]).normalize();
            append_quad(&mut mesh.textured,p.map(|p|p.to_array()),normal.to_array(),
                [[u0+(u1-u0)*a,v0+(v1-v0)*c],[u0+(u1-u0)*a,v0+(v1-v0)*d],
                 [u0+(u1-u0)*b,v0+(v1-v0)*d],[u0+(u1-u0)*b,v0+(v1-v0)*c]],TEXTURED_SHADE);
        }}
        claimed.insert(i);
        mesh.authored_cells[i]=Some("den:whirlpool");
    }
    claimed
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn den_whirlpool_funnel_matches_water_edges_without_changing_footing() {
        let g=GridGeometry{width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let tiles:Vec<_>=(0..16).map(|i|VisualTile {
            column:(i%4) as u32,row:(i/4) as u32,texture:Default::default(),priority:false,
            animation_frames:Some(vec![Handle::default(),Handle::default()].into()),
            source:VisualTileSource{tileset_id:"johto".into(),metatile_id:0x07,
                subtile_column:(i%4) as u8,subtile_row:(i/4) as u8,
                tile_index:crate::cave_water::DEN_WHIRLPOOL_DRAWING[i]},
        }).collect();
        let cells=tiles.iter().collect::<Vec<_>>();
        let mut mesh=TerrainMeshData::default();mesh.authored_cells=vec![None;16];mesh.footing_heights=vec![0.;16];
        assert_eq!(append(&mut mesh,&cells,&g).len(),4);
        assert_eq!(mesh.footing_heights,vec![0.;16]);
        assert!(mesh.textured.positions.contains(&[8.,(crate::profile::WATER_HEIGHT - 1.6),8.]));
        assert!(mesh.textured.positions.iter().all(|p|p[0]>=0. && p[0]<=16. && p[2]>=0. && p[2]<=16.
            && p[1]>=(crate::profile::WATER_HEIGHT - 1.6) && p[1]<=crate::profile::WATER_HEIGHT));
        assert!(mesh.textured.positions.iter().filter(|p|p[0]==0. || p[0]==16. || p[2]==0. || p[2]==16.)
            .all(|p|p[1]==crate::profile::WATER_HEIGHT));
        assert!(mesh.textured.normals.iter().all(|n|Vec3::from_array(*n).is_finite() && n[1]>0.));
        let mut integrated=TerrainMeshData::default();integrated.authored_cells=vec![None;16];integrated.footing_heights=vec![0.;16];
        for row in 0..4 {for col in 0..4 {
            let (w,e,n,s)=g.bounds(col,row);
            append_top(&mut integrated.textured,[w,e,n,s],0.,g.uv(col,row));
        }}
        let old=integrated.textured.positions.len() as u32;
        assert!(dragons_den::finish(&mut integrated,"DragonsDenB1F",&cells,&g,[0,0]));
        assert!(integrated.textured.indices.iter().all(|&i|
            i>=old || ![0,1,4,5].contains(&(i as usize/4))),
            "the original flat whirlpool planes must no longer render");
        assert!(integrated.textured.indices.iter().any(|&i|i>=old));
        assert_eq!(integrated.footing_heights,vec![0.;16]);
    }
}
