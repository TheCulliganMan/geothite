//! Complete native wall courses reveal the player without removing their floor.
//! Source ownership/collision stay separate from camera-dependent presentation.
use super::*;

fn eligible(p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    let Form::RocketWall {blue,..}=p.form else{return false;};
    p.width==1&&p.height==1&&p.front_rows==1.&&p.depth_pixels==8.
        &&p.rise_pixels==crate::rocket_base::WALL_HEIGHT
        &&p.column<g.width&&p.row<g.height&&p.ground<cells.len()&&blue<cells.len()
        &&crate::rocket_base::is_wall_cell("TeamRocketBaseB1F",&cells[p.row*g.width+p.column].source)
        &&crate::rocket_base::is_lower_face(&cells[blue].source)
        &&cells[p.ground].source.tileset_id.as_ref()=="underground"
        &&cells[p.ground].source.tile_index==crate::rocket_base::FLOOR_TILE
}

fn same_material(a:&Placement,b:&Placement)->bool {
    let (Form::RocketWall {blue:a_blue,..},Form::RocketWall {blue:b_blue,..})=(a.form,b.form)
        else{return false;};
    a_blue==b_blue&&a.ground==b.ground&&a.rise_pixels==b.rise_pixels
}

fn courses(placements:&[Placement],cells:&[&VisualTile],g:&GridGeometry)->Vec<Vec<usize>> {
    if g.width==0||g.height==0||cells.len()!=g.width*g.height {return Vec::new();}
    let mut owner=vec![usize::MAX;cells.len()];
    for (i,p) in placements.iter().enumerate().filter(|(_,p)|eligible(p,cells,g)) {
        let cell=p.row*g.width+p.column;
        if owner[cell]!=usize::MAX {return Vec::new();}
        owner[cell]=i;
    }
    let mut used=vec![false;placements.len()];let mut result=Vec::new();
    for y in 0..g.height {for x in 0..g.width {
        let first=owner[y*g.width+x];if first==usize::MAX||used[first] {continue;}
        let available=|x:usize,y:usize| {
            let i=owner[y*g.width+x];i!=usize::MAX&&!used[i]&&same_material(&placements[first],&placements[i])
        };
        let mut width=1;while x+width<g.width&&available(x+width,y) {width+=1;}
        let mut height=1;
        while y+height<g.height&&(x..x+width).all(|x|available(x,y+height)) {height+=1;}
        let mut group=Vec::with_capacity(width*height);
        for row in y..y+height {for column in x..x+width {
            let i=owner[row*g.width+column];used[i]=true;group.push(i);
        }}
        result.push(group);
    }}
    result
}

pub(in crate::mesh) fn append_all(mesh:&mut TerrainMeshData,g:&GridGeometry,placements:&[Placement],
    cells:&[&VisualTile],origin:[i32;2]) {
    let groups=courses(placements,cells,g);
    if groups.is_empty() {
        cave_boulder_fields::append_all(mesh,g,placements,cells,origin);return;
    }
    let mut owner=vec![usize::MAX;cells.len()];let mut grouped=vec![false;placements.len()];
    for (group,indices) in groups.iter().enumerate() {for &i in indices {
        let p=&placements[i];owner[p.row*g.width+p.column]=group;grouped[i]=true;
        // Reuse the ordinary append's exact floor restoration, omitting only
        // the deferred upright body. Ground never belongs to a reveal range.
        append_internal(mesh,g,p,cells,origin,false);
    }}
    let others=placements.iter().enumerate().filter(|(i,_)|!grouped[*i])
        .map(|(_,p)|p.clone()).collect::<Vec<_>>();
    cave_boulder_fields::append_all(mesh,g,&others,cells,origin);
    for (group,indices) in groups.iter().enumerate() {
        let first=mesh.textured.positions.len();
        for &i in indices {
            let p=&placements[i];let (x,y)=(p.column,p.row);
            // Keep buried faces at independently fading course boundaries.
            // A revealed neighbor must never expose an uncapped wall end.
            let open=[y==0||owner[(y-1)*g.width+x]!=group,
                x+1==g.width||owner[y*g.width+x+1]!=group,
                y+1==g.height||owner[(y+1)*g.width+x]!=group,
                x==0||owner[y*g.width+x-1]!=group];
            let Form::RocketWall {blue,..}=p.form else{unreachable!()};
            append_rocket_wall_body(&mut mesh.textured,g,p,open,blue);
        }
        mesh.textured.cutaway_ranges.push(first..mesh.textured.positions.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap,BTreeSet};

    fn fixture()->(GridGeometry,Vec<VisualTile>,Vec<Placement>) {
        let g=GridGeometry {width:8,height:6,tile_width:8.,tile_height:8.,origin_x:-32.,origin_z:-24.};
        let mut tiles=(0..48).map(|i|VisualTile {column:(i%8)as u32,row:(i/8)as u32,
            texture:Default::default(),animation_frames:None,priority:false,
            source:VisualTileSource {tileset_id:"underground".into(),metatile_id:1,
                subtile_column:(i%4)as u8,subtile_row:(i/8%4)as u8,tile_index:0x10},
        }).collect::<Vec<_>>();
        tiles[47].source.tile_index=0x14;
        let placements=[(2,1),(3,1),(4,1),(2,2),(3,2),(4,2),(2,3),(3,3)].into_iter()
            .map(|(column,row)| {
                tiles[row*8+column].source.tile_index=4;
                Placement {column,row,width:1,height:1,ground:0,
                    form:Form::RocketWall {open:[true;4],blue:47},rise_pixels:32.,
                    depth_pixels:8.,front_rows:1.}
            }).collect();
        (g,tiles,placements)
    }

    #[test]
    fn rocket_wall_reveal_courses_partition_only_owned_rectangles_and_reject_ambiguity() {
        let (g,mut tiles,p)=fixture();let cells=tiles.iter().collect::<Vec<_>>();
        let groups=courses(&p,&cells,&g);assert_eq!(groups.len(),2);
        assert_eq!(groups.iter().flatten().copied().collect::<BTreeSet<_>>(),(0..p.len()).collect());
        for group in &groups {
            let x0=group.iter().map(|&i|p[i].column).min().unwrap();
            let x1=group.iter().map(|&i|p[i].column).max().unwrap();
            let y0=group.iter().map(|&i|p[i].row).min().unwrap();
            let y1=group.iter().map(|&i|p[i].row).max().unwrap();
            assert_eq!(group.len(),(x1-x0+1)*(y1-y0+1),"no reveal object bridges an empty floor cell");
        }
        let mut duplicates=p.clone();duplicates.push(p[0].clone());
        assert!(courses(&duplicates,&cells,&g).is_empty(),"ambiguous source owners cannot acquire reveal ownership");
        let mut wrong=p.clone();wrong[0].rise_pixels=48.;
        assert!(courses(&wrong,&cells,&g).iter().all(|group|!group.contains(&0)));
        tiles[p[0].row*8+p[0].column].source.tileset_id="cave".into();
        assert!(courses(&p,&tiles.iter().collect::<Vec<_>>(),&g).iter().all(|group|!group.contains(&0)));
    }

    #[test]
    fn rocket_wall_reveal_keeps_floor_opaque_and_closes_every_course_end() {
        let (g,tiles,p)=fixture();let cells=tiles.iter().collect::<Vec<_>>();
        let mut mesh=TerrainMeshData {footing_heights:vec![6.25;48],..Default::default()};
        append_all(&mut mesh,&g,&p,&cells,[0,0]);
        assert_eq!(mesh.footing_heights,vec![6.25;48]);assert_eq!(mesh.textured.cutaway_ranges.len(),2);
        let mut floor_area=0.;
        for tri in mesh.textured.indices.chunks_exact(3) {
            let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
            if points.iter().all(|p|p.y==0.)&&tri.iter().all(|&i|mesh.textured.normals[i as usize][1]>0.99) {
                assert!(tri.iter().all(|&i|mesh.textured.cutaway_ranges.iter().all(|range|!range.contains(&(i as usize)))));
                floor_area+=(points[1]-points[0]).cross(points[2]-points[0]).length()*0.5;
            }
        }
        assert!((floor_area-p.len()as f32*64.).abs()<0.001);
        for range in &mesh.textured.cutaway_ranges {
            let mut lo=Vec3::splat(f32::INFINITY);let mut hi=Vec3::splat(f32::NEG_INFINITY);
            for &point in &mesh.textured.positions[range.clone()] {let point=Vec3::from_array(point);lo=lo.min(point);hi=hi.max(point);}
            assert_eq!(lo.y,0.);assert_eq!(hi.y,32.,"reveal never lowers the ceiling-height wall");
            let mut faces=[0.;5];
            for tri in mesh.textured.indices.chunks_exact(3).filter(|tri|range.contains(&(tri[0]as usize))) {
                assert!(tri.iter().all(|&i|range.contains(&(i as usize))));
                let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                let normal=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);
                let axis=if normal.y>0.99 {0}else if normal.x>0.99 {1}else if normal.x< -0.99 {2}
                    else if normal.z>0.99 {3}else{4};
                assert!((b-a).cross(c-a).dot(normal)>0.);
                faces[axis]+=(b-a).cross(c-a).length()*0.5;
            }
            let size=hi-lo;
            assert_eq!(faces,[size.x*size.z,size.z*32.,size.z*32.,size.x*32.,size.x*32.],
                "caps and both buried ends remain closed when an adjoining course fades");
        }
    }

    fn triangles(mesh:&SurfaceMeshData)->BTreeMap<Vec<u32>,usize> {
        let mut result=BTreeMap::new();
        for tri in mesh.indices.chunks_exact(3) {
            let mut key=Vec::with_capacity(36);
            for &i in tri {let i=i as usize;
                key.extend(mesh.positions[i].into_iter().chain(mesh.normals[i]).chain(mesh.uvs[i])
                    .chain(mesh.colors[i]).map(f32::to_bits));
            }
            *result.entry(key).or_default()+=1;
        }
        result
    }

    #[test]
    fn external_rocket_wall_reveal_preserves_all_existing_surfaces_source_and_support() {
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(
            std::path::PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"))).unwrap();
        let pack=loaded.pack();let map=&pack.data().maps["TeamRocketBaseB1F"];
        let art=&pack.runtime_files()["data/tilesets/underground_metatiles.bin"];
        let (w,h)=(map.attributes.width as usize*4,map.attributes.height as usize*4);
        let tiles=(0..w*h).map(|i|{let (x,y)=(i%w,i/w);let block=map.blocks[y/4*(w/4)+x/4];
            VisualTile {column:x as u32,row:y as u32,texture:Default::default(),animation_frames:None,priority:false,
                source:VisualTileSource {tileset_id:"underground".into(),metatile_id:block,
                    subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,
                    tile_index:art[block as usize*16+y%4*4+x%4]as u16}}
        }).collect::<Vec<_>>();
        let original=tiles.clone();let cells=tiles.iter().collect::<Vec<_>>();
        let g=GridGeometry {width:w,height:h,tile_width:8.,tile_height:8.,origin_x:-32.,origin_z:-16.};
        let p=resolve("TeamRocketBaseB1F",&cells,&g,None);let groups=courses(&p,&cells,&g);
        let walls=p.iter().filter(|p|matches!(p.form,Form::RocketWall {..})).count();
        assert!(walls>100&&groups.len()<walls/4,"use complete courses instead of a draw object per source tile");
        assert_eq!(groups.iter().map(Vec::len).sum::<usize>(),walls);
        let initial=TerrainMeshData {footing_heights:vec![7.25;w*h],authored_cells:vec![None;w*h],..Default::default()};
        let mut baseline=initial.clone();cave_boulder_fields::append_all(&mut baseline,&g,&p,&cells,[0,0]);
        let mut revealed=initial.clone();append_all(&mut revealed,&g,&p,&cells,[0,0]);
        assert_eq!(tiles,original);assert_eq!(revealed.footing_heights,initial.footing_heights);
        assert_eq!(revealed.authored_cells,initial.authored_cells);assert_eq!(revealed.solid,baseline.solid);
        assert_eq!(revealed.textured.cutaway_ranges.len(),baseline.textured.cutaway_ranges.len()+groups.len());
        let before=triangles(&baseline.textured);let after=triangles(&revealed.textured);
        for (surface,count) in before {
            assert!(after.get(&surface).copied().unwrap_or(0)>=count,
                "every prior triangle corner, normal and source pigment must remain bit-identical");
        }
        assert!(revealed.textured.indices.len()>baseline.textured.indices.len(),"independent courses retain their buried end faces");
        println!("Rocket reveal: {walls} source wall cells in {} closed courses; five statue/backing objects remain separate",groups.len());
    }
}
