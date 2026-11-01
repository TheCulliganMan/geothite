//! Closed, low water-boundary floats for complete Dragon's Den source blocks.
use super::*;
use std::collections::HashSet;

fn drawing(id: u16) -> Option<[u16;16]> {
    Some(match id {
        0x07 => crate::cave_water::DEN_WHIRLPOOL_DRAWING,
        0x31 => [0x58,0x58,0x58,0x58,0x58,0x58,0x58,0x58,0x14,0x14,0x14,0x14,0x14,0x14,0x14,0x14],
        0x34 => [0x58,0x58,0x14,0x14,0x58,0x58,0x14,0x14,0x58,0x58,0x14,0x14,0x58,0x58,0x14,0x14],
        0x36 => [0x14,0x14,0x58,0x58,0x14,0x14,0x58,0x58,0x14,0x14,0x58,0x58,0x14,0x14,0x58,0x58],
        0x39 => [0x14,0x14,0x14,0x14,0x14,0x14,0x14,0x14,0x58,0x58,0x58,0x58,0x58,0x58,0x58,0x58],
        0x3a => [0x14,0x14,0x58,0x58,0x14,0x14,0x58,0x58,0x58,0x58,0x58,0x58,0x58,0x58,0x58,0x58],
        _ => return None,
    })
}

fn resolve(cells: &[&VisualTile], g: &GridGeometry) -> Vec<(usize,usize)> {
    let mut selected = Vec::new();
    for row in 0..g.height.saturating_sub(3) {
        for col in 0..g.width.saturating_sub(3) {
            let s=&cells[row*g.width+col].source;
            if s.tileset_id.as_ref() != "johto" || s.subtile_column != 0 || s.subtile_row != 0 {continue;}
            let Some(d)=drawing(s.metatile_id) else {continue;};
            if !(0..16).all(|j| {
                let t=&cells[(row+j/4)*g.width+col+j%4].source;
                t.tileset_id==s.tileset_id && t.metatile_id==s.metatile_id
                    && t.subtile_column as usize==j%4 && t.subtile_row as usize==j/4 && t.tile_index==d[j]
            }) {continue;}
            let Some(water)=(0..16).filter(|&j|d[j]==0x14).map(|j|(row+j/4)*g.width+col+j%4)
                .find(|&i|cells[i].animation_frames.as_ref().is_some_and(|f|f.len()>1 && f.contains(&cells[i].texture)))
                else {continue;};
            selected.extend((0..16).filter(|&j|d[j]==0x58).map(|j|((row+j/4)*g.width+col+j%4,water)));
        }
    }
    selected
}

fn triangle(out: &mut SurfaceMeshData, p: [Vec3;3], g: &GridGeometry, origin: [i32;2], tint: [f32;4]) {
    let cross=(p[1]-p[0]).cross(p[2]-p[0]);
    if cross.length_squared()<1e-8 {return;}
    let normal=cross.normalize();
    let light=Vec3::new(-0.35,0.85,0.4).normalize();
    let shade=0.67+0.33*normal.dot(light).max(0.);
    let first=out.positions.len() as u32;
    for p in p {
        out.positions.push(p.to_array());out.normals.push(normal.to_array());
        out.uvs.push(cave_cliff::stone_uv(p,g,origin,false));
        let mut color=cave_cliff::pigment(tint,p,g,origin,false);
        for c in &mut color[..3] {*c *= shade;}
        out.colors.push(color);
    }
    out.indices.extend([first,first+1,first+2]);
}

pub(super) fn append(mesh: &mut TerrainMeshData, cells: &[&VisualTile], g: &GridGeometry,
    origin: [i32;2]) -> HashSet<usize> {
    let selected=resolve(cells,g);
    let claimed:HashSet<_>=selected.iter().map(|&(i,_)|i).collect();
    let base=crate::profile::WATER_HEIGHT*g.tile_height/SOURCE_TILE_HEIGHT;
    for (i,water) in selected {
        let col=i%g.width;let row=i/g.width;
        let (w,e,n,s)=g.bounds(col,row);
        append_top(&mut mesh.textured,[w,e,n,s],base,g.uv(water%g.width,water/g.width));
        let center=Vec3::new((w+e)*0.5,base,(n+s)*0.5);
        // Tapered closed floats have a wet lower band and a worn mineral cap.
        // Their entire silhouette stays inside the original eight-pixel slot.
        let rings=[(-0.06,0.23),(0.,0.34),(0.27,0.29),(0.43,0.10),(0.46,0.)];
        let at=|ring:usize,j:usize| {
            let angle=j as f32*std::f32::consts::TAU/12.;
            center+Vec3::new(angle.cos()*rings[ring].1*g.tile_width,
                rings[ring].0*g.tile_height,angle.sin()*rings[ring].1*g.tile_height)
        };
        for ring in 0..rings.len()-1 {for j in 0..12 {
            let p=[at(ring,j),at(ring+1,j),at(ring+1,(j+1)%12),at(ring,(j+1)%12)];
            let color=if ring==0 {[0.16,0.20,0.20,1.]} else {[0.32,0.35,0.32,1.]};
            triangle(&mut mesh.solid,[p[0],p[1],p[2]],g,origin,color);
            triangle(&mut mesh.solid,[p[0],p[2],p[3]],g,origin,color);
        }}
        for j in 0..12 {triangle(&mut mesh.solid,[center+Vec3::Y*(-0.06*g.tile_height),at(0,j),at(0,(j+1)%12)],g,origin,[0.16,0.20,0.20,1.]);}
        // Connect only actual adjacent boundary cells; never bridge a gap.
        for (dx,dz) in [(1,0),(0,1)] {
            if col+dx>=g.width || row+dz>=g.height || !claimed.contains(&(i+dx+dz*g.width)) {continue;}
            let along=Vec3::new(dx as f32*g.tile_width,0.,dz as f32*g.tile_height);
            let side=Vec3::new(-along.z,0.,along.x).normalize()*g.tile_width*0.055;
            let a=center+along*0.30+Vec3::Y*g.tile_height*0.12;
            let b=center+along*0.70+Vec3::Y*g.tile_height*0.12;
            let up=Vec3::Y*g.tile_height*0.045;
            let faces=[[a-side-up,b-side-up,b+side-up,a+side-up],
                [a-side+up,a+side+up,b+side+up,b-side+up],
                [a-side-up,a-side+up,b-side+up,b-side-up],
                [a+side-up,b+side-up,b+side+up,a+side+up]];
            for p in faces {triangle(&mut mesh.solid,[p[0],p[1],p[2]],g,origin,[0.16,0.19,0.18,1.]);
                triangle(&mut mesh.solid,[p[0],p[2],p[3]],g,origin,[0.16,0.19,0.18,1.]);}
        }
        mesh.authored_cells[i]=Some("den:water-boundary");
    }
    claimed
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_boundary_builds_closed_floats_and_keeps_water_and_support_datums() {
        let g=GridGeometry {width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let d=drawing(0x31).unwrap();
        let mut tiles:Vec<_>=(0..16).map(|i| VisualTile {column:(i%4) as u32,row:(i/4) as u32,
            source:VisualTileSource {tileset_id:"johto".into(),metatile_id:0x31,subtile_column:(i%4) as u8,
                subtile_row:(i/4) as u8,tile_index:d[i]},texture:Default::default(),
            animation_frames:(d[i]==0x14).then(||vec![Default::default(),Default::default()].into()),priority:false}).collect();
        let mut mesh=TerrainMeshData {authored_cells:vec![None;16],footing_heights:vec![0.;16],..Default::default()};
        let selected=append(&mut mesh,&tiles.iter().collect::<Vec<_>>(),&g,[0,0]);
        assert_eq!(selected.len(),8);assert!(selected.iter().all(|&i|i<8));
        assert_eq!(mesh.footing_heights,vec![0.;16]);
        assert!(mesh.textured.positions.iter().all(|p|p[1]==crate::profile::WATER_HEIGHT));
        assert!(mesh.solid.positions.iter().all(|p|p[0]>=0. && p[0]<=32. && p[2]>=0. && p[2]<=16.
            && p[1]>=crate::profile::WATER_HEIGHT-0.49 && p[1]<=crate::profile::WATER_HEIGHT+3.69));
        assert!(mesh.solid.normals.iter().all(|n|Vec3::from_array(*n).is_finite()));
        // The first float has closed top and bottom and outward side normals.
        assert!(mesh.solid.normals[..288].iter().any(|n|n[1]>0.9));
        assert!(mesh.solid.normals[..288].iter().any(|n|n[1] < -0.99));
        let mut edges=std::collections::BTreeMap::new();
        for tri in mesh.solid.positions[..288].chunks_exact(3) {
            let p:[ [i32;3];3 ]=std::array::from_fn(|i|tri[i].map(|v|(v*100000.).round() as i32));
            for (a,b) in [(0,1),(1,2),(2,0)] {
                let mut edge=[p[a],p[b]];edge.sort();
                *edges.entry(edge).or_insert(0)+=1;
            }
        }

        assert!(edges.values().all(|&uses|uses==2), "the float must be a closed volume");
        tiles[0].source.tile_index=0x14;
        assert!(resolve(&tiles.iter().collect::<Vec<_>>(),&g).is_empty());
        tiles[0].source.tile_index=0x58;
        for t in &mut tiles {t.animation_frames=None;}
        assert!(resolve(&tiles.iter().collect::<Vec<_>>(),&g).is_empty());
    }
}
