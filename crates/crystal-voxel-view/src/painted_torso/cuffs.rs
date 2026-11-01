//! A continuous cloth/skin boundary on the finished, weighted arm surface.
use super::{has_short_sleeves, SleevePaint};
use crate::{mesh::SurfaceMeshData, npc_appearance::Look};
use bevy::prelude::Vec3;
use std::collections::{BTreeMap, HashMap};

const END: f32 = 0.74;
const EPSILON: f32 = 0.000001;
type Point = [i32;3];
type Edge = [Point;2];
fn key(p: [f32;3]) -> Point { p.map(|v| (v*100_000.).round() as i32) }
fn edge(a: [f32;3], b: [f32;3]) -> Edge {
    let (a,b)=(key(a),key(b));[a.min(b),a.max(b)]
}
fn distance(p: [f32;3]) -> f32 {
    let d=p[0].abs()-END;
    if d.abs()<=EPSILON {0.} else {d}
}
fn material(mesh: &SurfaceMeshData, face: [usize;3]) -> Option<bool> {
    if !face.iter().all(|&i| {
        let p=mesh.positions[i];
        (0.4..1.2).contains(&p[0].abs()) && (1.93..2.66).contains(&p[1])
    }) {return None;}
    if face.iter().all(|&i|mesh.uvs[i][0]<0.625) {Some(true)}
    else if face.iter().all(|&i|mesh.uvs[i][0]>0.80) {Some(false)}
    else {None}
}

#[derive(Clone)]
struct Cut {
    point: [f32;3],
    joints: [u16;4],
    weights: [f32;4],
}

fn skin(ids: &[[u16;4]], weights: &[[f32;4]], a: usize, b: usize, t: f32)
    -> ([u16;4],[f32;4]) {
    let mut blend=BTreeMap::<u16,f32>::new();
    for (i,s) in [(a,1.-t),(b,t)] {
        for slot in 0..4 {
            if weights[i][slot]>0. {*blend.entry(ids[i][slot]).or_default()+=weights[i][slot]*s;}
        }
    }
    let mut blend: Vec<_>=blend.into_iter().filter(|&(_,w)|w>0.).collect();
    // The fitted cuff has upper-arm/forearm influences. A cut must not drop
    // an influence merely to fit the four-slot transport format.
    assert!(blend.len()<=4,"cuff cut exceeds its original skin field");
    blend.sort_by(|a,b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    let total: f32=blend.iter().map(|p|p.1).sum();
    (std::array::from_fn(|i|blend.get(i).map_or(0,|p|p.0)),
        std::array::from_fn(|i|blend.get(i).map_or(0.,|p|p.1/total)))
}

fn append_flag(mesh: &mut SurfaceMeshData, i: usize, hidden: bool) {
    if !hidden {return;}
    if let Some(last)=mesh.cutaway_ranges.last_mut().filter(|r|r.end==i) {last.end=i+1;}
    else {mesh.cutaway_ranges.push(i..i+1);}
}

fn split_vertex(mesh: &mut SurfaceMeshData, ids: &mut Vec<[u16;4]>, weights: &mut Vec<[f32;4]>,
    a: usize, b: usize, cut: &Cut) -> u32 {
    let t=(END-mesh.positions[a][0].abs()) /
        (mesh.positions[b][0].abs()-mesh.positions[a][0].abs());
    let hidden=mesh.cutaway_ranges.iter().any(|r|r.contains(&a));
    assert_eq!(hidden,mesh.cutaway_ranges.iter().any(|r|r.contains(&b)),
        "cuff cut crosses an unrelated hidden-body boundary");
    let i=mesh.positions.len();
    let normal=Vec3::from_array(mesh.normals[a]).lerp(Vec3::from_array(mesh.normals[b]),t).normalize();
    let color=std::array::from_fn(|s|mesh.colors[a][s]*(1.-t)+mesh.colors[b][s]*t);
    let uv=std::array::from_fn(|s|mesh.uvs[a][s]*(1.-t)+mesh.uvs[b][s]*t);
    mesh.positions.push(cut.point);mesh.normals.push(normal.to_array());
    mesh.colors.push(color);mesh.uvs.push(uv);
    ids.push(cut.joints);weights.push(cut.weights);append_flag(mesh,i,hidden);
    i as u32
}

fn remap(mesh: &mut SurfaceMeshData, ids: &mut Vec<[u16;4]>, weights: &mut Vec<[f32;4]>,
    index: u32, old: bool, new: bool, look: Look, cache: &mut HashMap<(u32,bool),u32>) -> u32 {
    if old==new {return index;}
    *cache.entry((index,new)).or_insert_with(|| {
        let a=index as usize;let i=mesh.positions.len();
        let hidden=mesh.cutaway_ranges.iter().any(|r|r.contains(&a));
        let original=SleevePaint::for_surface(look,old).position(mesh.uvs[a]);
        let uv=SleevePaint::for_surface(look,new).sample(original);
        mesh.positions.push(mesh.positions[a]);mesh.normals.push(mesh.normals[a]);
        mesh.colors.push(mesh.colors[a]);mesh.uvs.push(uv);
        ids.push(ids[a]);weights.push(weights[a]);append_flag(mesh,i,hidden);
        i as u32
    })
}

fn triangulate(mesh: &mut SurfaceMeshData, polygon: &[u32]) {
    let area=|f: [u32;3]| {
        let p=f.map(|i|Vec3::from_array(mesh.positions[i as usize]));
        (p[1]-p[0]).cross(p[2]-p[0]).length_squared()
    };
    match polygon {
        [a,b,c] => mesh.indices.extend([*a,*b,*c]),
        [a,b,c,d] => {
            let first=[[*a,*b,*c],[*a,*c,*d]];
            let second=[[*b,*c,*d],[*b,*d,*a]];
            let faces=if first.map(area).into_iter().fold(f32::INFINITY,f32::min)
                >= second.map(area).into_iter().fold(f32::INFINITY,f32::min) {first} else {second};
            for f in faces {mesh.indices.extend(f);}
        }
        _ => panic!("invalid cuff polygon"),
    }
}

pub(crate) fn finish(mesh: &mut SurfaceMeshData, ids: &mut Vec<[u16;4]>,
    weights: &mut Vec<[f32;4]>, look: Look) {
    if !has_short_sleeves(look) {return;}
    #[cfg(all(feature="people-review",not(target_arch="wasm32")))]
    if std::env::var_os("CRYSTAL_PEOPLE_REVIEW_CUFF_BASELINE").is_some() {return;}
    let original=std::mem::take(&mut mesh.indices);
    let mut cuts=HashMap::<Edge,Cut>::new();
    for f in original.chunks_exact(3) {
        let face=[f[0] as usize,f[1] as usize,f[2] as usize];
        if material(mesh,face).is_none() {continue;}
        for (a,b) in [(face[0],face[1]),(face[1],face[2]),(face[2],face[0])] {
            if distance(mesh.positions[a])*distance(mesh.positions[b])>=0. {continue;}
            let k=edge(mesh.positions[a],mesh.positions[b]);
            cuts.entry(k).or_insert_with(|| {
                let (a,b)=if key(mesh.positions[a])<=key(mesh.positions[b]) {(a,b)} else {(b,a)};
                let t=(END-mesh.positions[a][0].abs())/
                    (mesh.positions[b][0].abs()-mesh.positions[a][0].abs());
                let mut p=Vec3::from_array(mesh.positions[a]).lerp(Vec3::from_array(mesh.positions[b]),t).to_array();
                p[0]=mesh.positions[a][0].signum()*END;
                let (j,w)=skin(ids,weights,a,b,t);
                Cut{point:p,joints:j,weights:w}
            });
        }
    }
    let mut vertices=HashMap::<[u32;2],u32>::new();
    let mut painted=HashMap::<(u32,bool),u32>::new();
    for f in original.chunks_exact(3) {
        let face=[f[0],f[1],f[2]];
        let old=material(mesh,face.map(|i|i as usize));
        let mut mids=[None;3];
        for e in 0..3 {
            let (a,b)=(face[e],face[(e+1)%3]);
            let k=edge(mesh.positions[a as usize],mesh.positions[b as usize]);
            if let Some(cut)=cuts.get(&k) {
                mids[e]=Some(*vertices.entry([a.min(b),a.max(b)]).or_insert_with(||
                    split_vertex(mesh,ids,weights,a as usize,b as usize,cut)));
            }
        }
        if let Some(old)=old {
            let d=face.map(|i|distance(mesh.positions[i as usize]));
            if d.iter().all(|&d|d==0.) {mesh.indices.extend(face);continue;}
            for cloth in [true,false] {
                let mut polygon=Vec::with_capacity(4);
                for i in 0..3 {
                    if if cloth {d[i]<=0.} else {d[i]>=0.} {polygon.push(face[i]);}
                    if d[i]*d[(i+1)%3]<0. {polygon.push(mids[i].expect("joined cuff intersection"));}
                }
                if polygon.len()<3 {continue;}
                let mapped: Vec<_>=polygon.into_iter().map(|i|remap(mesh,ids,weights,i,old,cloth,look,&mut painted)).collect();
                triangulate(mesh,&mapped);
            }
        } else {
            // Split adjacent material copies too. A T-junction can look shut
            // in bind pose and open after the continuous skin field is fitted.
            match mids.iter().filter(|m|m.is_some()).count() {
                0 => mesh.indices.extend(face),
                1 => {
                    let e=mids.iter().position(Option::is_some).unwrap();
                    let [a,b,c]=std::array::from_fn(|i|face[(e+i)%3]);let ab=mids[e].unwrap();
                    triangulate(mesh,&[a,ab,c]);triangulate(mesh,&[ab,b,c]);
                }
                2 => {
                    let e=(mids.iter().position(Option::is_none).unwrap()+1)%3;
                    let [a,b,c]=std::array::from_fn(|i|face[(e+i)%3]);
                    let (ab,bc)=(mids[e].unwrap(),mids[(e+1)%3].unwrap());
                    triangulate(mesh,&[b,bc,ab]);triangulate(mesh,&[a,ab,bc,c]);
                }
                3 => {
                    let [a,b,c]=face;let [ab,bc,ca]=mids.map(Option::unwrap);
                    for f in [[a,ab,ca],[ab,b,bc],[ca,bc,c],[ab,bc,ca]] {triangulate(mesh,&f);}
                }
                _=>unreachable!(),
            }
        }
    }
}

#[cfg(any(test, all(feature="people-review",not(target_arch="wasm32"))))]
pub(crate) mod validation {
    use super::*;
    use bevy::prelude::Mat4;
    use bevy::render::mesh::VertexAttributeValues;
    use std::collections::BTreeSet;
    type Position=[u32;3];
    type Boundary=BTreeMap<[Position;2],Vec<(bool,usize,usize)>>;
    fn position(p: [f32;3]) -> Position {p.map(|v|if v==0. {0} else {v.to_bits()})}
    fn boundary(points: &[[f32;3]],uvs: &[[f32;2]],indices: &[u32]) -> Boundary {
        let mut all=Boundary::new();
        for f in indices.chunks_exact(3) {
            let f=[f[0] as usize,f[1] as usize,f[2] as usize];
            if !f.iter().all(|&i|(0.4..1.2).contains(&points[i][0].abs())
                &&(1.93..2.66).contains(&points[i][1])) {continue;}
            let cloth=if f.iter().all(|&i|uvs[i][0]<0.625) {true}
                else if f.iter().all(|&i|uvs[i][0]>0.80) {false} else {continue;};
            for (a,b) in [(f[0],f[1]),(f[1],f[2]),(f[2],f[0])] {
                let (ka,kb)=(position(points[a]),position(points[b]));
                all.entry([ka.min(kb),ka.max(kb)]).or_default().push((cloth,a,b));
            }
        }
        all.retain(|_,uses|uses.iter().any(|u|u.0)&&uses.iter().any(|u|!u.0));all
    }

    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    pub(crate) fn validate() {
        supplied_short_cuffs_are_continuous_closed_and_share_their_moving_skin();
        cuff_cut_splits_other_uv_islands_without_opening_the_surface();
    }

    #[cfg_attr(test,test)]
    pub(crate) fn cuff_cut_splits_other_uv_islands_without_opening_the_surface() {
        let mut mesh=SurfaceMeshData::default();let mut ids=Vec::new();let mut weights=Vec::new();
        let corners=[[0.68,2.2,0.],[0.80,2.2,0.],[0.68,2.4,0.],[0.80,2.4,0.]];
        let look=crate::npc_appearance::look(crate::new_bark_actors::character_meshes::CharacterKind::Nurse);
        for (face,eligible) in [([0,1,2],true),([1,3,2],false)] {
            for i in face {
                let p=corners[i];let n=mesh.positions.len() as u32;
                mesh.positions.push(p);mesh.normals.push([0.,0.,1.]);
                mesh.colors.push([p[0],p[1],0.5,1.]);
                mesh.uvs.push(if eligible {SleevePaint::for_surface(look,false).sample(p)}
                    else {[0.65+(p[0]-0.68),0.2+(p[1]-2.2)]});
                mesh.indices.push(n);ids.push([0,1,0,0]);
                let w=0.2+(p[0]-0.68)*4.;weights.push([1.-w,w,0.,0.]);
            }
        }
        // Retain an unrelated hidden lower-body surface and its flags.
        for p in [[0.,1.,0.],[0.1,1.,0.],[0.,1.1,0.]] {
            let n=mesh.positions.len() as u32;mesh.positions.push(p);mesh.normals.push([0.,0.,1.]);
            mesh.colors.push([p[0],p[1],0.5,1.]);mesh.uvs.push([0.9,0.3]);
            mesh.indices.push(n);ids.push([0,1,0,0]);weights.push([1.,0.,0.,0.]);
        }
        mesh.cutaway_ranges.push(6..9);
        let original=mesh.clone();let original_ids=ids.clone();let original_weights=weights.clone();
        finish(&mut mesh,&mut ids,&mut weights,look);
        let count=original.positions.len();
        assert_eq!(&mesh.positions[..count],&original.positions);
        assert_eq!(&mesh.normals[..count],&original.normals);
        assert_eq!(&mesh.colors[..count],&original.colors);
        assert_eq!(&mesh.uvs[..count],&original.uvs);
        assert_eq!(&ids[..count],&original_ids);assert_eq!(&weights[..count],&original_weights);
        assert_eq!(mesh.cutaway_ranges,original.cutaway_ranges);
        assert_eq!(&mesh.indices[mesh.indices.len()-3..],&[6,7,8]);
        let seam=boundary(&mesh.positions,&mesh.uvs,&mesh.indices);
        assert!(!seam.is_empty(),"the requested cloth/skin boundary was not cut");
        for points in seam.keys() {
            assert!(points.iter().all(|p|(f32::from_bits(p[0])-END).abs()<EPSILON));
        }
        let mut edges=BTreeMap::<[Position;2],usize>::new();let mut area=0.;
        for f in mesh.indices.chunks_exact(3) {
            if f.iter().all(|&i|mesh.positions[i as usize][1]<2.) {continue;}
            let p=[f[0],f[1],f[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
            let cross=(p[1]-p[0]).cross(p[2]-p[0]);
            assert!(cross.z>0. && cross.x==0. && cross.y==0.,"cuff cut changed winding/plane");
            area+=cross.z*0.5;
            for (a,b) in [(0,1),(1,2),(2,0)] {
                let (a,b)=(position(p[a].to_array()),position(p[b].to_array()));
                *edges.entry([a.min(b),a.max(b)]).or_default()+=1;
            }
        }
        assert!((area-0.024_f32).abs()<0.000001,"cuff cut changed covered surface area");
        for ([a,b],count) in edges {
            if count==2 {continue;}
            let (a,b)=(a.map(f32::from_bits),b.map(f32::from_bits));
            assert_eq!(count,1);
            assert!((a[0]==b[0]&&[0.68,0.80].contains(&a[0]))
                || (a[1]==b[1]&&[2.2,2.4].contains(&a[1])),"new open interior edge: {a:?} {b:?}");
        }
        for frame in 0..16 {
            let moving=Mat4::from_rotation_z(frame as f32*0.07)
                * Mat4::from_translation(Vec3::new(0.1,0.05,0.));
            let mut shared=BTreeMap::<Position,Vec3>::new();
            for i in 0..mesh.positions.len() {
                let p=Vec3::from_array(mesh.positions[i]);
                let posed=(0..4).map(|s|if ids[i][s]==0 {p*weights[i][s]}
                    else {moving.transform_point3(p)*weights[i][s]}).sum::<Vec3>();
                if let Some(old)=shared.insert(position(p.to_array()),posed) {
                    assert!(old.distance(posed)<0.000001,"cuff UV copies separated in motion");
                }
                assert!((mesh.colors[i][0]-p.x).abs()<0.000001);
                assert!((mesh.colors[i][1]-p.y).abs()<0.000001);
                assert_eq!(mesh.colors[i][3],1.);
            }
        }
    }

    #[cfg_attr(test,test)]
    #[cfg_attr(test,ignore="requires supplied rigs and character paintings")]
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    pub(crate) fn supplied_short_cuffs_are_continuous_closed_and_share_their_moving_skin() {
        use crate::new_bark_actors::character_meshes::CharacterKind::*;
        let mut total=0;
        for kind in [Red,Misty,Brock,PokefanM,Nurse] {
            let look=crate::npc_appearance::look(kind);
            let rig=crate::open_people::get(look.base).unwrap();
            let prepared=crate::open_people_skin::prepare_appearance(rig,kind);
            let mesh=prepared.review_body();
            let Some(VertexAttributeValues::Float32x3(p))=mesh.attribute(bevy::prelude::Mesh::ATTRIBUTE_POSITION) else {panic!("positions")};
            let Some(VertexAttributeValues::Float32x2(uv))=mesh.attribute(bevy::prelude::Mesh::ATTRIBUTE_UV_0) else {panic!("UVs")};
            let Some(VertexAttributeValues::Uint16x4(ids))=mesh.attribute(bevy::prelude::Mesh::ATTRIBUTE_JOINT_INDEX) else {panic!("joints")};
            let Some(VertexAttributeValues::Float32x4(w))=mesh.attribute(bevy::prelude::Mesh::ATTRIBUTE_JOINT_WEIGHT) else {panic!("weights")};
            let indices:Vec<_>=mesh.indices().unwrap().iter().map(|i|i as u32).collect();
            let seam=boundary(p,uv,&indices);assert!(seam.len()>64,"{kind:?}: missing cuff perimeter");
            let mut adjacent=[BTreeMap::<Position,BTreeSet<Position>>::new(),BTreeMap::new()];
            let mut fields=BTreeMap::<Position,([u16;4],[f32;4])>::new();
            for ([a,b],uses) in &seam {
                assert_eq!(uses.len(),2,"{kind:?}: cuff edge lacks a matching material face");
                for v in [a,b] {
                    let q=v.map(f32::from_bits);
                    assert!((q[0].abs()-END).abs()<EPSILON*2.,"{kind:?}: stepped cuff at {q:?}");
                }
                let side=usize::from(f32::from_bits(a[0])<0.);
                adjacent[side].entry(*a).or_default().insert(*b);
                adjacent[side].entry(*b).or_default().insert(*a);
                for &(_,i,j) in uses {
                    for v in [i,j] {
                        let current=(ids[v],w[v]);
                        if let Some(old)=fields.insert(position(p[v]),current) {
                            assert_eq!(old,current,"{kind:?}: material copies use different moving skin");
                        }
                    }
                }
            }
            for graph in &adjacent {
                assert!(graph.len()>32,"{kind:?}: missing a sleeve side");
                assert!(graph.values().all(|v|v.len()==2),"{kind:?}: branched/open cuff loop");
                let mut visited=BTreeSet::new();let mut pending=vec![*graph.keys().next().unwrap()];
                while let Some(p)=pending.pop() {
                    if visited.insert(p) {pending.extend(graph[&p].iter().copied());}
                }
                assert_eq!(visited.len(),graph.len(),"{kind:?}: disconnected cuff fragments");
            }
            total+=seam.len();
        }
        eprintln!("Short cuff checks: five people, ten complete cloth/skin loops, {total} shared edges; planar boundaries and identical moving skin on every material copy.");
    }
}
