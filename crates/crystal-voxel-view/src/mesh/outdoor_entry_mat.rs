//! Complete native outdoor thresholds share the existing woven cloth finish.
//! Only pigment/UV ownership changes; submitted floor triangles stay exact.
use super::*;

const LABEL: &str = "outdoor/entry-cloth";
fn local(s: &VisualTileSource) -> Option<[usize;2]> {
    if s.tileset_id.as_ref()=="park" && s.tile_index==0x4f && s.subtile_column<4 && s.subtile_row<4 {
        match s.metatile_id {
            0x0a if s.subtile_row>=2 => return Some([s.subtile_column as usize,(s.subtile_row-2)as usize]),
            0x0b if s.subtile_column>=2 => return Some([(s.subtile_column-2)as usize,s.subtile_row as usize]),
            _=>{},
        }
    }
    if s.tileset_id.as_ref()=="kanto" && s.tile_index==0x4f && (2..4).contains(&s.subtile_row) {
        match (s.metatile_id,s.subtile_column) {
            (0x46,2..=3)=>return Some([(s.subtile_column-2)as usize,(s.subtile_row-2)as usize]),
            (0x4a,0..=1)=>return Some([(s.subtile_column+2)as usize,(s.subtile_row-2)as usize]),
            _=>{},
        }
    }
    if s.tileset_id.as_ref()=="kanto" && s.metatile_id==0x04 && s.tile_index==0x4f
        && s.subtile_column<4 && (2..4).contains(&s.subtile_row) {
        return Some([s.subtile_column as usize,(s.subtile_row-2)as usize]);
    }
    if s.tileset_id.as_ref()=="battle_tower_outside" && s.metatile_id==0x0c
        && s.tile_index==0x40 && s.subtile_column<4 && (2..4).contains(&s.subtile_row) {
        return Some([s.subtile_column as usize,(s.subtile_row-2)as usize]);
    }
    if s.tileset_id.as_ref()=="kanto" && s.tile_index==0x4f && s.subtile_row<4 {
        return match (s.metatile_id,s.subtile_column) {
            (0x17,2..=3)=>Some([(s.subtile_column-2)as usize,s.subtile_row as usize]),
            (0x16,0..=1)=>Some([s.subtile_column as usize,s.subtile_row as usize]),
            _=>None,
        };
    }
    if s.tileset_id.as_ref()=="johto" && s.tile_index==0x40 && s.subtile_row<4 {
        match (s.metatile_id,s.subtile_column) {
            (0x0b,0..=1)=>return Some([s.subtile_column as usize,s.subtile_row as usize]),
            (0x0e,2..=3)=>return Some([(s.subtile_column-2)as usize,s.subtile_row as usize]),
            _=>{},
        }
    }
    if s.tileset_id.as_ref()!="johto" || s.tile_index!=0x40
        || !(2..4).contains(&s.subtile_row) {return None;}
    let x=match (s.metatile_id,s.subtile_column) {
        (0x0c,2..=3)=>s.subtile_column-2,
        (0x0d,0..=1)=>s.subtile_column+2,
        _=>return None,
    };
    Some([x as usize,(s.subtile_row-2) as usize])
}
pub(super) fn kanto_approach(cells:&[&VisualTile],g:&GridGeometry,x:usize,y:usize,west:bool)->bool {
    x+4<=g.width && y+4<=g.height && (0..16).all(|j| {
        let s=&cells[(y+j/4)*g.width+x+j%4].source;
        let tile=if (j%4>=2)==west {0x4f}
            else if west && j==4 || !west && j==14 {0x39} else {0x23};
        s.tileset_id.as_ref()=="kanto" && s.metatile_id==if west {0x17}else{0x16}
            && s.subtile_column as usize==j%4 && s.subtile_row as usize==j/4 && s.tile_index==tile
    })
}
fn kanto_front_approach(cells:&[&VisualTile],g:&GridGeometry,x:usize,y:usize)->bool {
    x+4<=g.width && y+4<=g.height && (0..16).all(|j| {
        let s=&cells[(y+j/4)*g.width+x+j%4].source;
        let tile=if j>=8 {0x4f}else if j==4 {0x39}else{0x23};
        s.tileset_id.as_ref()=="kanto" && s.metatile_id==0x04
            && s.subtile_column as usize==j%4 && s.subtile_row as usize==j/4 && s.tile_index==tile
    })
}
fn johto_side_approach(cells:&[&VisualTile],g:&GridGeometry,x:usize,y:usize,west:bool)->bool {
    x+4<=g.width && y+4<=g.height && (0..16).all(|j| {
        let s=&cells[(y+j/4)*g.width+x+j%4].source;
        s.tileset_id.as_ref()=="johto" && s.metatile_id==if west {0x0e}else{0x0b}
            && s.subtile_column as usize==j%4 && s.subtile_row as usize==j/4
            && s.tile_index==if (j%4>=2)==west {0x40}else{6}
    })
}
fn pigment(tile:&VisualTile, images:&TerrainImageSamples)->Option<[f32;4]> {
    let rgba=tile_rgba(images,tile).ok()?;
    if rgba.len()!=256 || rgba.chunks_exact(4).any(|p|p[3]!=255) {return None;}
    let colors=rgba.chunks_exact(4).map(|p|[p[0],p[1],p[2]])
        .collect::<std::collections::BTreeSet<_>>();
    if !(2..=4).contains(&colors.len()) {return None;}
    let mut color=[0.,0.,0.,1.];
    for p in rgba.chunks_exact(4) {for c in 0..3 {
        let s=f32::from(p[c])/255.;
        color[c]+=if s<=0.04045 {s/12.92}else{((s+0.055)/1.055).powf(2.4)}/64.;
    }}
    Some(color)
}
fn complete(cells:&[&VisualTile], g:&GridGeometry, x:usize,y:usize)->bool {
    x+8<=g.width && y+4<=g.height && (0..4).all(|dy|(0..8).all(|dx| {
        let s=&cells[(y+dy)*g.width+x+dx].source;
        let rail=dx<2 || dx>=6;
        let tile=if dy<2 {6}else if rail {if dy==2 {0x5a}else{0x59}}else{0x40};
        s.tileset_id.as_ref()=="johto" && s.metatile_id==if dx<4 {0x0c}else{0x0d}
            && s.subtile_column==(dx%4) as u8 && s.subtile_row==dy as u8 && s.tile_index==tile
    }))
}
fn kanto_cave_approach(cells:&[&VisualTile],g:&GridGeometry,x:usize,y:usize)->bool {
    x+8<=g.width && y+4<=g.height && (0..32).all(|j| {
        let (dx,dy)=(j%8,j/8);
        let s=&cells[(y+dy)*g.width+x+dx].source;
        let tile=if dy<2 {if dy==1 && dx%4==0 {0x39}else{0x23}}
            else if (2..6).contains(&dx) {0x4f}
            else {(if dy==2 {0x2a}else{0x3a})+(dx%2)as u16};
        s.tileset_id.as_ref()=="kanto" && s.metatile_id==if dx<4 {0x46}else{0x4a}
            && s.subtile_column as usize==dx%4 && s.subtile_row as usize==dy
            && s.tile_index==tile
    })
}
pub(super) fn north_approach(cells:&[&VisualTile],g:&GridGeometry,p:BuildingPlacement)->bool {
    p.width==8 && p.row.checked_sub(4).is_some_and(|y|complete(cells,g,p.column,y))
}
fn floor_vertices(mesh:&SurfaceMeshData, faces:&[usize], g:&GridGeometry, i:usize)->Option<Vec<u32>> {
    if faces.len()!=2 {return None;}
    let ids=faces.iter().flat_map(|&f|mesh.indices[f*3..f*3+3].iter().copied())
        .collect::<std::collections::BTreeSet<_>>().into_iter().collect::<Vec<_>>();
    if ids.len()!=4 {return None;}
    let (w,e,n,s)=g.bounds(i%g.width,i/g.width);
    let (u0,u1,v0,v1)=g.uv(i%g.width,i/g.width);
    let mut corners=[false;4];
    for &id in &ids {
        let k=id as usize;let p=mesh.positions[k];let uv=mesh.uvs[k];
        // The native mat may follow a gentle terrace contour. Transferring
        // its existing triangles and normals must not flatten that surface.
        if mesh.normals[k][1]<=0. || !mesh.normals[k].iter().all(|v|v.is_finite())
            || !p.iter().all(|v|v.is_finite())
            || mesh.cutaway_ranges.iter().any(|r|r.contains(&k)) {return None;}
        let corner=[([w,n],[u0,v0]),([w,s],[u0,v1]),([e,s],[u1,v1]),([e,n],[u1,v0])]
            .iter().position(|(point,expected)| (p[0]-point[0]).abs()<0.0001
                && (p[2]-point[1]).abs()<0.0001 && uv==*expected)?;
        corners[corner]=true;
    }
    corners.into_iter().all(|v|v).then_some(ids)
}

pub(super) fn finish(mesh:&mut TerrainMeshData,cells:&[&VisualTile],g:&GridGeometry,
    images:&TerrainImageSamples) {
    if cells.len()!=g.width*g.height || mesh.authored_cells.len()!=cells.len()
        || !g.tile_width.is_finite() || !g.tile_height.is_finite()
        || g.tile_width<=0. || g.tile_height<=0.
        || !cells.iter().any(|t|local(&t.source).is_some()) {return;}
    let mut faces=vec![Vec::new();cells.len()];
    for (number,tri) in mesh.textured.indices.chunks_exact(3).enumerate() {
        let uv=[tri[0],tri[1],tri[2]].map(|i|mesh.textured.uvs[i as usize]);
        if let Some(i)=new_bark::sampled_cell(&uv,g).filter(|&i|local(&cells[i].source).is_some()) {
            faces[i].push(number);
        }
    }
    let mut removed=std::collections::BTreeSet::new();
    for y in 0..g.height {for x in 0..g.width {
        let s=&cells[y*g.width+x].source;
        if s.subtile_column!=0 || s.subtile_row!=0 {continue;}
        let (dx,dy,width,height)=if s.tileset_id.as_ref()=="johto" && s.metatile_id==0x0c
            && complete(cells,g,x,y) {(2,2,4,2)}
            else if s.tileset_id.as_ref()=="park" {
                let Some([dx,dy,width,height])=super::modeled_exteriors::park_gate::threshold(cells,g,x,y)
                    else {continue;};
                (dx,dy,width,height)
            }
            else if s.tileset_id.as_ref()=="kanto" && s.metatile_id==0x46
                && kanto_cave_approach(cells,g,x,y) {(2,2,4,2)}
            else if s.tileset_id.as_ref()=="kanto" && s.metatile_id==0x04
                && kanto_front_approach(cells,g,x,y) {(0,2,4,2)}
            else if s.tileset_id.as_ref()=="battle_tower_outside" && s.metatile_id==0x0c
                && x>=2 && super::modeled_exteriors::tower_gate::matches(cells,g,
                    BuildingPlacement{column:x-2,row:y+4,width:8,height:8,roof_rows:4,ground_tile_index:6}) {
                (0,2,4,2)
            }
            else if s.tileset_id.as_ref()=="kanto" && matches!(s.metatile_id,0x16|0x17)
                && kanto_approach(cells,g,x,y,s.metatile_id==0x17) {
                (if s.metatile_id==0x17 {2}else{0},0,2,4)
            } else if s.tileset_id.as_ref()=="johto" && matches!(s.metatile_id,0x0b|0x0e)
                && johto_side_approach(cells,g,x,y,s.metatile_id==0x0e) {
                (if s.metatile_id==0x0e {2}else{0},0,2,4)
            } else {continue;};
        let plan=(0..height).flat_map(|row|(0..width).map(move|col|(y+dy+row)*g.width+x+dx+col))
            .map(|i|Some((i,mesh.authored_cells[i].is_none().then_some(())?,
                floor_vertices(&mesh.textured,&faces[i],g,i)?,pigment(cells[i],images)?)))
            .collect::<Option<Vec<_>>>();
        let Some(plan)=plan else {continue;};
        let fabric_base=if width==2 && height==4 {
            crate::scenery_finish::SOURCE_FABRIC_SIDE_BASE
        } else {crate::scenery_finish::SOURCE_FABRIC_BASE};
        for (i,_,ids,color) in plan {
            let offset=mesh.solid.positions.len() as u32;
            let [lx,ly]=local(&cells[i].source).unwrap();
            let (w,_,n,_)=g.bounds(i%g.width,i/g.width);
            for &id in &ids {
                let k=id as usize;let p=mesh.textured.positions[k];
                mesh.solid.positions.push(p);
                mesh.solid.normals.push(mesh.textured.normals[k]);
                mesh.solid.colors.push(std::array::from_fn(|c|color[c]*mesh.textured.colors[k][c]));
                mesh.solid.uvs.push([fabric_base[0]
                    +lx as f32*8.+(p[0]-w)/g.tile_width*8.,
                    fabric_base[1]
                    +ly as f32*8.+(p[2]-n)/g.tile_height*8.]);
            }
            for &face in &faces[i] {
                for id in &mesh.textured.indices[face*3..face*3+3] {
                    mesh.solid.indices.push(offset+ids.iter().position(|v|v==id).unwrap() as u32);
                }
                removed.insert(face);
            }
            mesh.authored_cells[i]=Some(LABEL);
        }
    }}
    if !removed.is_empty() {
        mesh.textured.indices=mesh.textured.indices.chunks_exact(3).enumerate()
            .filter(|(i,_)|!removed.contains(i)).flat_map(|(_,tri)|tri.iter().copied()).collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paired_cave_carpet_finishes_only_the_complete_native_threshold() {
        let (mut tiles,g,original,images)=fixture();
        for (i,tile) in tiles.iter_mut().enumerate() {
            let (x,y)=(i%8,i/8);
            tile.source.tileset_id="kanto".into();
            tile.source.metatile_id=if x<4 {0x46}else{0x4a};
            tile.source.tile_index=if y<2 {if y==1 && x%4==0 {0x39}else{0x23}}
                else if (2..6).contains(&x) {0x4f}
                else {(if y==2 {0x2a}else{0x3a})+(x%2)as u16};
        }
        let before=tiles.clone();
        let mut out=original.clone();finish(&mut out,&tiles.iter().collect::<Vec<_>>(),&g,&images);
        assert_eq!(tiles,before,"the native drawing is authoritative");
        assert_eq!(out.footing_heights,original.footing_heights);
        assert_eq!(out.textured.positions,original.textured.positions);
        assert_eq!(out.textured.indices.len()+out.solid.indices.len(),original.textured.indices.len());
        for (i,owner) in out.authored_cells.iter().enumerate() {
            assert_eq!(*owner,((i/8>=2)&&(2..6).contains(&(i%8))).then_some(LABEL),
                "only the central carpet is owned; cave glyphs retain their producer");
        }
        let base=crate::scenery_finish::SOURCE_FABRIC_BASE;
        for (&p,&uv) in out.solid.positions.iter().zip(&out.solid.uvs) {
            assert_eq!(uv,[base[0]+p[0]-16.,base[1]+p[2]-16.],"one continuous shared weave");
        }
        for i in 0..tiles.len() {
            let mut changed=tiles.clone();changed[i].source.tile_index^=1;
            let mut rejected=original.clone();finish(&mut rejected,&changed.iter().collect::<Vec<_>>(),&g,&images);
            assert_eq!(rejected,original,"partial source pair retains fallback: {i}");
        }
        let mut custom=original.clone();custom.authored_cells[18]=Some("custom");
        let before=custom.clone();finish(&mut custom,&tiles.iter().collect::<Vec<_>>(),&g,&images);
        assert_eq!(custom,before,"custom ownership is retained");
    }
    #[test]
    fn kanto_front_gate_cloth_keeps_native_contour_and_complete_source_ownership() {
        let sources=(0..16).map(|j| {
            let mut s=super::super::tests::source_with_tile(0x04,(j%4)as u8,(j/4)as u8,
                if j>=8 {0x4f}else if j==4 {0x39}else{0x23});
            s.tileset_id="kanto".into();s
        }).collect();
        let tiles=super::super::tests::frame(4,4,sources).tiles;
        let g=GridGeometry{width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut original=TerrainMeshData{authored_cells:vec![None;16],footing_heights:vec![3.25;16],..Default::default()};
        let mut images=TerrainImageSamples::default();
        for (i,tile) in tiles.iter().enumerate() {
            let (w,e,n,s)=g.bounds(i%4,i/4);
            append_top(&mut original.textured,[w,e,n,s],3.25,g.uv(i%4,i/4));
            images.pixels.insert(tile.texture.id(),TileImageSample::Rgba((0..64).flat_map(|p|
                if p%2==0 {[220,60,50,255]}else{[250,120,100,255]}).collect()));
        }
        for (p,n) in original.textured.positions.iter_mut().zip(&mut original.textured.normals) {
            p[1]+=p[0]*0.001+p[2]*0.002;
            *n=Vec3::new(-0.001,1.,-0.002).normalize().to_array();
        }
        let refs=tiles.iter().collect::<Vec<_>>();let mut out=original.clone();finish(&mut out,&refs,&g,&images);
        assert_eq!(out.footing_heights,original.footing_heights);
        assert_eq!(out.textured.positions,original.textured.positions);
        assert!(out.authored_cells[..8].iter().all(Option::is_none));
        assert!(out.authored_cells[8..].iter().all(|v|*v==Some(LABEL)));
        let triangles=|mesh:&TerrainMeshData|[&mesh.textured,&mesh.solid].into_iter().flat_map(|m|
            m.indices.chunks_exact(3).map(|tri|tri.iter().map(|&i|(m.positions[i as usize],m.normals[i as usize])).collect::<Vec<_>>()))
            .collect::<Vec<_>>();
        let before=triangles(&original);let after=triangles(&out);
        assert_eq!(before.len(),after.len());assert!(before.iter().all(|t|after.contains(t)));
        let base=crate::scenery_finish::SOURCE_FABRIC_BASE;
        let lo=out.solid.uvs.iter().fold([f32::INFINITY;2],|a,p|[a[0].min(p[0]),a[1].min(p[1])]);
        let hi=out.solid.uvs.iter().fold([f32::NEG_INFINITY;2],|a,p|[a[0].max(p[0]),a[1].max(p[1])]);
        assert_eq!(lo,base);assert_eq!(hi,[base[0]+32.,base[1]+16.]);
        let mut night=images.clone();for sample in night.pixels.values_mut() {
            if let TileImageSample::Rgba(p)=sample {for pixel in p.chunks_exact_mut(4) {pixel[0]/=2;pixel[1]/=2;}}
        }
        let mut darker=original.clone();finish(&mut darker,&refs,&g,&night);
        assert_ne!(darker.solid.colors,out.solid.colors);assert_eq!(darker.solid.positions,out.solid.positions);
        for i in 0..16 {for fault in 0..4 {
            let mut changed=tiles.clone();match fault {
                0=>changed[i].source.tile_index^=1,1=>changed[i].source.metatile_id^=1,
                2=>changed[i].source.subtile_column^=1,_=>changed[i].source.tileset_id="johto".into(),
            }
            let mut rejected=original.clone();finish(&mut rejected,&changed.iter().collect::<Vec<_>>(),&g,&images);
            assert_eq!(rejected,original,"a changed complete drawing must retain fallback");
        }}
        let mut custom=original.clone();custom.authored_cells[8]=Some("custom");
        let before=custom.clone();finish(&mut custom,&refs,&g,&images);assert_eq!(custom,before);
    }

    #[test]
    fn side_gate_mats_keep_native_floor_coverage_and_require_complete_drawings() {
        for (atlas,west) in ["kanto","johto"].into_iter().flat_map(|atlas|[true,false].map(|west|(atlas,west))) {
            let sources=(0..16).map(|j| {
                let tile=if (j%4>=2)==west {if atlas=="kanto" {0x4f}else{0x40}}
                    else if atlas=="johto" {6}
                    else if west && j==4 || !west && j==14 {0x39}else{0x23};
                let block=if atlas=="kanto" {if west {0x17}else{0x16}}else if west {0x0e}else{0x0b};
                let mut s=super::super::tests::source_with_tile(block,
                    (j%4)as u8,(j/4)as u8,tile);s.tileset_id=atlas.into();s
            }).collect();
            let tiles=super::super::tests::frame(4,4,sources).tiles;
            let g=GridGeometry {width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let mut original=TerrainMeshData {authored_cells:vec![None;16],footing_heights:vec![2.;16],..Default::default()};
            let mut images=TerrainImageSamples::default();
            for (i,tile) in tiles.iter().enumerate() {
                let (w,e,n,s)=g.bounds(i%4,i/4);append_top(&mut original.textured,[w,e,n,s],2.,g.uv(i%4,i/4));
                images.pixels.insert(tile.texture.id(),TileImageSample::Rgba((0..64).flat_map(|p|
                    if p%2==0 {[220,60,50,255]}else{[250,120,100,255]}).collect()));
            }
            let mut out=original.clone();finish(&mut out,&tiles.iter().collect::<Vec<_>>(),&g,&images);
            assert_eq!(out.footing_heights,original.footing_heights);
            assert_eq!(out.textured.positions,original.textured.positions);
            assert_eq!(out.authored_cells.iter().filter(|v|**v==Some(LABEL)).count(),8);
            let triangles=|mesh:&TerrainMeshData|[&mesh.solid,&mesh.textured].into_iter().flat_map(|m|
                m.indices.chunks_exact(3).map(|tri|[tri[0],tri[1],tri[2]].map(|i|m.positions[i as usize]))).collect::<Vec<_>>();
            let before=triangles(&original);let after=triangles(&out);
            assert_eq!(before.len(),after.len());assert!(before.iter().all(|t|after.contains(t)));
            assert!(out.solid.uvs.iter().all(|uv|crate::scenery_finish::source_fabric_coordinates(*uv).is_some()));
            let base=crate::scenery_finish::SOURCE_FABRIC_SIDE_BASE;
            let lo=out.solid.uvs.iter().fold([f32::INFINITY;2],|a,p|[a[0].min(p[0]),a[1].min(p[1])]);
            let hi=out.solid.uvs.iter().fold([f32::NEG_INFINITY;2],|a,p|[a[0].max(p[0]),a[1].max(p[1])]);
            assert_eq!(lo,base);assert_eq!(hi,[base[0]+16.,base[1]+32.],"side mats share one complete weave/binding");
            let mut contoured=original.clone();
            for (p,n) in contoured.textured.positions.iter_mut().zip(&mut contoured.textured.normals) {
                p[1]=2.+p[0]*0.001+p[2]*0.002+p[0]*p[2]*0.00001;
                *n=Vec3::new(-0.001-p[2]*0.00001,1.,-0.002-p[0]*0.00001).normalize().to_array();
            }
            let before=contoured.clone();finish(&mut contoured,&tiles.iter().collect::<Vec<_>>(),&g,&images);
            assert_eq!(contoured.authored_cells.iter().filter(|v|**v==Some(LABEL)).count(),8,
                "a contoured runtime mat receives the same shared cloth finish");
            assert_eq!(contoured.footing_heights,before.footing_heights);
            let old=triangles(&before);let new=triangles(&contoured);
            assert_eq!(old.len(),new.len());assert!(old.iter().all(|t|new.contains(t)));
            for (p,n) in contoured.solid.positions.iter().zip(&contoured.solid.normals) {
                let i=before.textured.positions.iter().position(|v|v==p).unwrap();
                assert_eq!(*n,before.textured.normals[i],"the contour's original normal stays exact");
            }
            for failure in 0..3 {
                let mut changed=tiles.clone();let mut rejected=original.clone();
                if failure==0 {changed[0].source.tile_index^=1;}
                if failure==1 {changed[0].source.subtile_column=1;}
                if failure==2 {rejected.authored_cells[if west {2}else{0}]=Some("custom");}
                let before=rejected.clone();finish(&mut rejected,&changed.iter().collect::<Vec<_>>(),&g,&images);
                assert_eq!(rejected,before,"partial and custom side approaches retain native floor");
            }
        }
    }
    fn fixture()->(Vec<VisualTile>,GridGeometry,TerrainMeshData,TerrainImageSamples) {
        let sources=(0..32).map(|i| {
            let (x,y)=(i%8,i/8);
            let tile=if y<2 {6}else if x<2 || x>=6 {if y==2 {0x5a}else{0x59}}else{0x40};
            super::super::tests::source_with_tile(if x<4 {0x0c}else{0x0d},(x%4)as u8,y as u8,tile)
        }).collect::<Vec<_>>();
        let tiles=super::super::tests::frame(8,4,sources).tiles;
        let g=GridGeometry{width:8,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut mesh=TerrainMeshData{authored_cells:vec![None;32],footing_heights:vec![3.25;32],..Default::default()};
        for i in 0..32 {let (w,e,n,s)=g.bounds(i%8,i/8);
            append_top(&mut mesh.textured,[w,e,n,s],3.25,g.uv(i%8,i/8));}
        let mut images=TerrainImageSamples::default();
        let pixels=(0..64).flat_map(|i|if i%2==0 {[220,60,50,255]}else{[250,120,100,255]}).collect::<Vec<_>>();
        for tile in &tiles {images.pixels.insert(tile.texture.id(),TileImageSample::Rgba(pixels.clone()));}
        (tiles,g,mesh,images)
    }
    #[test]
    fn source_entry_cloth_preserves_floor_triangles_support_and_live_pigment() {
        let (tiles,g,original,images)=fixture();let cells=tiles.iter().collect::<Vec<_>>();
        let mut mesh=original.clone();finish(&mut mesh,&cells,&g,&images);
        assert_eq!(mesh.footing_heights,original.footing_heights);
        assert_eq!(mesh.textured.positions,original.textured.positions);
        assert_eq!(mesh.solid.indices.len()+mesh.textured.indices.len(),original.textured.indices.len());
        let before=original.textured.indices.chunks_exact(3).map(|tri|
            [tri[0],tri[1],tri[2]].map(|i|original.textured.positions[i as usize])).collect::<Vec<_>>();
        let after=[&mesh.textured,&mesh.solid].into_iter().flat_map(|m|m.indices.chunks_exact(3).map(|tri|
            [tri[0],tri[1],tri[2]].map(|i|m.positions[i as usize]))).collect::<Vec<_>>();
        assert_eq!(before.len(),after.len());assert!(before.iter().all(|tri|after.contains(tri)));
        assert_eq!(mesh.authored_cells.iter().filter(|v|**v==Some(LABEL)).count(),8);
        assert!(mesh.solid.uvs.iter().all(|uv|crate::scenery_finish::source_fabric_coordinates(*uv).is_some()));
        assert!(mesh.solid.positions.iter().all(|p|p[1]==3.25));
        let mut night=images.clone();
        for tile in &tiles {night.pixels.insert(tile.texture.id(),TileImageSample::Rgba((0..64).flat_map(|i|
            if i%2==0 {[60,20,55,255]}else{[100,40,90,255]}).collect()));}
        let mut dark=original.clone();finish(&mut dark,&cells,&g,&night);
        assert_eq!(mesh.solid.positions,dark.solid.positions);assert_ne!(mesh.solid.colors,dark.solid.colors);
        for rejected in 0..4 {
            let mut changed=tiles.clone();let mut out=original.clone();
            if rejected==0 {changed[0].source.tile_index^=1;}
            if rejected==1 {out.authored_cells[18]=Some("custom");}
            if rejected==2 {out.textured.indices.drain(18*6..18*6+3);}
            let before=out.clone();
            let unpainted=TerrainImageSamples::default();
            finish(&mut out,&changed.iter().collect::<Vec<_>>(),&g,
                if rejected==3 {&unpainted}else{&images});
            assert_eq!(out,before,"incomplete, custom, partial or unpainted mats stay untouched");
        }
        assert!(crate::scenery_finish::source_fabric_coordinates([82000.,2500.]).is_none());
        assert!(crate::scenery_finish::source_fabric_coordinates([85000.,0.]).is_none());
    }
}
