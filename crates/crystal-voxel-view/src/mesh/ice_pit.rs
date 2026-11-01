//! A real recessed shaft beneath the source hole's floor datum.
use super::*;

pub(super) fn append(out: &mut SurfaceMeshData, g: &GridGeometry, p: &Placement) {
    let (w,_,n,_)=g.bounds(p.column,p.row);
    let e=w+2.0*g.tile_width; let s=n+2.0*g.tile_height;
    let lip=g.tile_width*0.10;
    let [iw,ie,inorth,isouth]=[w+lip,e-lip,n+lip,s-lip];
    let ice=[0.36,0.65,0.72,1.0];
    for b in [[w,e,n,inorth],[w,e,isouth,s],[w,iw,inorth,isouth],[ie,e,inorth,isouth]] {
        append_solid_quad(out,[[b[0],0.,b[2]],[b[0],0.,b[3]],[b[1],0.,b[3]],[b[1],0.,b[2]]],
            [0.,1.,0.],ice);
    }
    let depth=-g.tile_height*1.5;
    append_solid_quad(out,[[iw,depth,inorth],[iw,depth,isouth],[ie,depth,isouth],[ie,depth,inorth]],
        [0.,1.,0.],[0.006,0.018,0.03,1.0]);
    for (a,b,normal) in [([iw,inorth],[ie,inorth],[0.,0.,1.]),
        ([ie,inorth],[ie,isouth],[-1.,0.,0.]),([ie,isouth],[iw,isouth],[0.,0.,-1.]),
        ([iw,isouth],[iw,inorth],[1.,0.,0.])] {
        for (top,bottom,color) in [(0.,-g.tile_height*0.2,[0.24,0.49,0.58,1.0]),
            (-g.tile_height*0.2,depth,[0.035,0.085,0.12,1.0])] {
            append_solid_quad(out,[[a[0],top,a[1]],[a[0],bottom,a[1]],
                [b[0],bottom,b[1]],[b[0],top,b[1]]],normal,color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shaft_has_depth_without_a_floor_bridge_or_support_mutation() {
        let g=GridGeometry {width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        for (block,ox,oy) in [(0x17,0,2),(0x1b,2,0)] {
            let mut t=(0..16).map(|i|VisualTile {column:(i%4) as u32,row:(i/4) as u32,
                animation_frames:None,priority:false,texture:Handle::default(),source:VisualTileSource {
                    tileset_id:"ice_path".into(),metatile_id:block,subtile_column:(i%4) as u8,
                    subtile_row:(i/4) as u8,tile_index:0x19 }}).collect::<Vec<_>>();
            for y in 0..2 {for x in 0..2 {t[(oy+y)*4+ox+x].source.tile_index=[[0x8e,0x8f],[0x9e,0x9f]][y][x];}}
            let refs=t.iter().collect::<Vec<_>>(); let groups=resolve("IcePathB1F",&refs,&g,None);
            assert_eq!(groups.len(),1); assert!(matches!(groups[0].form,Form::IcePit));
            let mut mesh=TerrainMeshData::default();mesh.footing_heights=vec![0.;16];
            super::super::append(&mut mesh,&g,&groups[0],&refs,[0,0]);
            assert!(mesh.textured.positions.is_empty());
            assert!(mesh.solid.positions.iter().all(|p|p[1]<=0.0));
            assert!(mesh.solid.positions.iter().any(|p|p[1]==-12.0));
            assert_eq!(mesh.footing_heights,vec![0.;16]);
            t[oy*4+ox].source.tile_index=0x19;
            assert!(resolve("IcePathB1F",&t.iter().collect::<Vec<_>>(),&g,None).is_empty());
        }
    }
}
