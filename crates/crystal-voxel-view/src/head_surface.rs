//! Round the supplied jaw and cheek silhouette while pinning facial landmarks.
use crate::{mesh::SurfaceMeshData, open_people::PersonRig};
use bevy::prelude::*;

/// The adventure player's face needs a continuous cheek and jaw surface.
/// Keep the crown and neck anchors for the existing cap and collar, while
/// retaining UV islands and weighted animation on the refined face.
pub(crate) fn adventure_profile(
    rig: &PersonRig,
    mesh: &mut SurfaceMeshData,
    ids: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
) {
    let is_head = |j: &[u16; 4], w: &[f32; 4]| {
        (0..4).filter(|&i| j[i] as usize == rig.head_joint)
            .map(|i| w[i]).sum::<f32>() > 0.5
    };
    let mut lo = Vec3::splat(f32::INFINITY);
    let mut hi = Vec3::splat(f32::NEG_INFINITY);
    for ((p, j), w) in mesh.positions.iter().zip(ids.iter()).zip(weights.iter()) {
        if is_head(j, w) {
            lo = lo.min(Vec3::from_array(*p));
            hi = hi.max(Vec3::from_array(*p));
        }
    }
    if !lo.is_finite() || !hi.is_finite() { return; }
    let height = hi.y - lo.y;
    for _ in 0..2 {
        let selected = mesh.positions.iter().zip(ids.iter()).zip(weights.iter())
            .map(|((p, j), w)| is_head(j, w)
                && p[1] > lo.y + height * 0.05
                && p[1] < hi.y - height * 0.08)
            .collect::<Vec<_>>();
        crate::hand_surface::smooth_selection(mesh, ids, weights, &selected);
    }
    let center = (lo + hi) * 0.5;
    for ((p, j), w) in mesh.positions.iter_mut().zip(ids.iter()).zip(weights.iter()) {
        if !is_head(j, w) || p[1] <= lo.y + height * 0.05 { continue; }
        let t = ((center.y - p[1]) / (height * 0.44)).clamp(0.0, 1.0);
        let t = t * t * (3.0 - 2.0 * t);
        p[0] = center.x + (p[0] - center.x) * (1.0 - 0.12 * t);
    }
    // A continuous cheek normal field avoids the source's dark jaw wedges.
    // Taper vertical curvature so the chin does not shade as a black underside.
    let half = (hi - lo) * 0.5;
    for i in 0..mesh.positions.len() {
        let p = mesh.positions[i];
        if is_head(&ids[i], &weights[i]) && p[1] > lo.y + height * 0.05
            && p[1] < hi.y - height * 0.08 {
            let normal = Vec3::new((p[0] - center.x) / half.x.powi(2),
                (p[1] - center.y) * 0.35 / half.y.powi(2),
                (p[2] - center.z) / half.z.powi(2));
            if normal.length_squared() > 1e-12 { mesh.normals[i] = normal.normalize().to_array(); }
        }
    }
}

pub(crate) fn smooth(
    rig: &PersonRig,
    mesh: &mut SurfaceMeshData,
    ids: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
) {
    let head: Vec<_> = ids
        .iter()
        .zip(weights.iter())
        .map(|(j, w)| {
            (0..4)
                .filter(|&i| j[i] as usize == rig.head_joint)
                .map(|i| w[i])
                .sum::<f32>()
                > 0.5
        })
        .collect();
    let mut lo = Vec3::splat(f32::INFINITY);
    let mut hi = Vec3::splat(f32::NEG_INFINITY);
    for (&p, &head) in mesh.positions.iter().zip(&head) {
        if head {
            lo = lo.min(Vec3::from_array(p));
            hi = hi.max(Vec3::from_array(p));
        }
    }
    if !lo.is_finite() || !hi.is_finite() {
        return;
    }
    let center = (lo + hi) * 0.5;
    let half = (hi - lo) * 0.5;
    let selected: Vec<_> = mesh
        .positions
        .iter()
        .zip(head)
        .map(|(&p, head)| {
            // Keep eye/nose/mouth placement and front accessory support authored.
            // The neck seam and crown also remain exact for collars and hats.
            head && p[1] > lo.y + half.y * 0.10
                && p[1] < center.y + half.y * 0.72
                && (p[2] < center.z + half.z * 0.55 || p[1] < center.y - half.y * 0.42)
        })
        .collect();
    crate::hand_surface::smooth_selection(mesh, ids, weights, &selected);
}

/// Give the player's drawn face a small anime nose and a receding chin in
/// profile. The eye plane, crown and mixed-weight neck seam stay authored.
pub(crate) fn player_profile(rig: &PersonRig, mesh: &mut SurfaceMeshData,
    ids: &[[u16; 4]], weights: &[[f32; 4]]) {
    let head_weight = |i: usize| (0..4)
        .filter(|&j| ids[i][j] as usize == rig.head_joint)
        .map(|j| weights[i][j]).sum::<f32>();
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for (i, &p) in mesh.positions.iter().enumerate() {
        if head_weight(i) > 0.5 {
            min = min.min(Vec3::from_array(p));
            max = max.max(Vec3::from_array(p));
        }
    }
    if !min.is_finite() || !max.is_finite() { return; }
    let center = (min + max) * 0.5;
    let half = (max - min) * 0.5;
    let displacement = |position: Vec3| {
        let local = (position - center) / half;
        if local.z <= 0.35 || local.y >= 0.18 { return 0.; }
        let front = ((local.z - 0.35) / 0.5).clamp(0., 1.);
        let front = front * front * (3. - 2. * front);
        let lower = (-local.y).clamp(0., 1.);
        let chin = -0.25 * lower * lower;
        let nose = 0.025 * (-(local.x / 0.20).powi(2)
            - ((local.y + 0.04) / 0.12).powi(2)).exp();
        let upper_fade = ((0.18 - local.y) / 0.10).clamp(0., 1.);
        half.z * front * (chin + nose) * upper_fade
    };
    for (i, p) in mesh.positions.iter_mut().enumerate() {
        if head_weight(i) < 0.85 { continue; }
        let original = Vec3::from_array(*p);
        let local = (original - center) / half;
        if local.z <= 0.35 || local.y >= 0.18 { continue; }
        p[2] += displacement(original);
        // Keep the authored smooth anime lighting field. Recomputing normals
        // around the tiny nose accent creates a rounded dark bump, while the
        // intended face reads through its drawn eyes, nose and mouth.
    }
}

/// Continuous rear-head lighting without moving the painted facial plane.
pub(crate) fn nape_normals(rig: &PersonRig, mesh: &mut SurfaceMeshData,
    ids: &[[u16;4]], weights: &[[f32;4]]) {
    let head = |i: usize| (0..4).filter(|&s| ids[i][s] as usize == rig.head_joint)
        .map(|s| weights[i][s]).sum::<f32>() > 0.5;
    let mut lo=Vec3::splat(f32::INFINITY);let mut hi=Vec3::splat(f32::NEG_INFINITY);
    for (i,p) in mesh.positions.iter().enumerate() {
        if head(i) {lo=lo.min(Vec3::from_array(*p));hi=hi.max(Vec3::from_array(*p));}
    }
    if !lo.is_finite() || !hi.is_finite() {return;}
    let center=(lo+hi)*0.5;let half=(hi-lo)*0.5;
    for (i,p) in mesh.positions.iter().enumerate() {
        if !head(i) || p[1]<=lo.y+(hi.y-lo.y)*0.05 {continue;}
        let fade=((center.z+half.z*0.55-p[2])/(half.z*0.45)).clamp(0.,1.);
        let target=Vec3::new((p[0]-center.x)/half.x.powi(2),
            (p[1]-center.y)*0.35/half.y.powi(2),(p[2]-center.z)/half.z.powi(2));
        if fade>0. && target.length_squared()>1e-12 {
            mesh.normals[i]=Vec3::from_array(mesh.normals[i])
                .lerp(target.normalize(),fade*fade*(3.-2.*fade)).normalize().to_array();
        }
    }
}

#[cfg(any(test, all(feature = "people-review", not(target_arch = "wasm32"))))]
pub(crate) mod review_validation {
    use super::*;
    pub(crate) fn validate() {
        for name in ["human-male", "human-female"] {
            let rig = crate::open_people::get(name).expect("profile review needs the supplied rig");
            let crate::open_people::RefinedBody { mut mesh, mut ids, mut weights } = rig.refined_body();
            adventure_profile(rig, &mut mesh, &mut ids, &mut weights);
            let before = mesh.clone();
            let old_ids = ids.clone(); let old_weights = weights.clone();
            let mut head_min = Vec3::splat(f32::INFINITY);
            let mut head_max = Vec3::splat(f32::NEG_INFINITY);
            for ((&p, ids), weights) in before.positions.iter().zip(&ids).zip(&weights) {
                let head: f32 = (0..4).filter(|&i| ids[i] as usize == rig.head_joint).map(|i| weights[i]).sum();
                if head > 0.5 { head_min = head_min.min(Vec3::from_array(p)); head_max = head_max.max(Vec3::from_array(p)); }
            }
            // Bound recession in the supplied head's anatomy. Fixed source
            // units impose different constraints on the two production rigs.
            let profile_limit = (head_max.z - head_min.z) * 0.125 + 0.00001;
            player_profile(rig, &mut mesh, &ids, &weights);
            assert_eq!(mesh.indices, before.indices);
            assert_eq!(mesh.uvs, before.uvs); assert_eq!(mesh.colors, before.colors);
            assert_eq!(mesh.normals, before.normals);
            assert_eq!(ids, old_ids); assert_eq!(weights, old_weights);
            let mut changed = Vec::new();
            for i in 0..mesh.positions.len() {
                let a = Vec3::from_array(before.positions[i]); let b = Vec3::from_array(mesh.positions[i]);
                assert_eq!(a.x.to_bits(), b.x.to_bits()); assert_eq!(a.y.to_bits(), b.y.to_bits());
                assert!(b.is_finite() && a.distance(b) <= profile_limit,
                    "{name}: profile displacement {} exceeds {profile_limit}", a.distance(b));
                let head: f32 = (0..4).filter(|&j| ids[i][j] as usize == rig.head_joint).map(|j| weights[i][j]).sum();
                if head < 0.85 { assert_eq!(before.positions[i], mesh.positions[i], "body or neck moved"); }
                if a.y >= head_max.y - (head_max.y - head_min.y) * 0.15 {
                    assert_eq!(before.positions[i], mesh.positions[i], "crown anchor moved");
                }
                if before.positions[i] != mesh.positions[i] { changed.push(i); }
            }
            assert!(changed.len() > 10, "profile has no visible geometry change");
            let mut moved = vec![false; mesh.positions.len()];
            for &i in &changed { moved[i] = true; }
            for face in mesh.indices.chunks_exact(3) {
                if !face.iter().any(|&i| moved[i as usize]) { continue; }
                let a = [face[0], face[1], face[2]].map(|i| Vec3::from_array(before.positions[i as usize]));
                let b = [face[0], face[1], face[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let old = (a[1] - a[0]).cross(a[2] - a[0]);
                let new = (b[1] - b[0]).cross(b[2] - b[0]);
                if old.length_squared() > 1e-18 {
                    assert!(old.dot(new) > 0. && new.length_squared() > old.length_squared() * 0.1,
                        "profile reversed or collapsed a face");
                }
            }
            let bounds = rig.locomotion_bounds();
            let gait = crate::human_locomotion::HumanLocomotion::new(rig);
            let mut pose = rig.nodes.clone();
            for run in [0., 0.5, 1.] { for step in 0..16 {
                gait.sample(rig, step as f32 / 16. * std::f32::consts::TAU, run, &mut pose);
                let inverse = crate::human_locomotion::world(rig, &pose, rig.mesh_node).inverse();
                let palette: Vec<_> = rig.skin_nodes.iter().zip(&rig.inverse_binds).map(|(&node, bind)|
                    inverse * crate::human_locomotion::world(rig, &pose, node) * *bind).collect();
                for &i in &changed {
                    let p = Vec3::from_array(mesh.positions[i]);
                    let skinned: Vec3 = (0..4).map(|j| palette[ids[i][j] as usize].transform_point3(p) * weights[i][j]).sum();
                    assert!(skinned.is_finite() && skinned.cmpge(bounds.0).all() && skinned.cmple(bounds.1).all(),
                        "profile leaves animated bounds");
                }
            }}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires supplied male and female rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn anime_profile_preserves_artwork_projection_body_and_animated_bounds() {
        super::review_validation::validate();
    }
    #[test]
    #[ignore = "requires supplied Kurt rig in CRYSTAL_OPEN_MODEL_ROOT"]
    fn kurt_nape_refinement_preserves_painted_front_and_collared_body() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig = PersonRig::parse(&std::fs::read(root.join("people/kenney/human-male.glb")).unwrap()).unwrap();
        let mut mesh = rig.mesh.clone();
        let mut ids = rig.ids.clone();
        let mut weights = rig.weights.clone();
        smooth(&rig, &mut mesh, &mut ids, &mut weights);
        bald_crown(&rig, &mut mesh, &mut ids, &mut weights,
            crate::npc_appearance::look(crate::new_bark_actors::character_meshes::CharacterKind::Kurt));
        let before = mesh.clone();
        let before_ids = ids.clone();
        let before_weights = weights.clone();
        let head = |i: usize| (0..4).filter(|&s| before_ids[i][s] as usize == rig.head_joint)
            .map(|s| before_weights[i][s]).sum::<f32>() > 0.5;
        let mut lo = Vec3::splat(f32::INFINITY);
        let mut hi = Vec3::splat(f32::NEG_INFINITY);
        for (i, p) in before.positions.iter().enumerate() {
            if head(i) { lo=lo.min(Vec3::from_array(*p));hi=hi.max(Vec3::from_array(*p)); }
        }
        let center=(lo+hi)*0.5;let half=(hi-lo)*0.5;
        for _ in 0..2 { smooth(&rig, &mut mesh, &mut ids, &mut weights); }
        nape_normals(&rig, &mut mesh, &ids, &weights);
        let mut protected_front=0;let mut rounded=0;
        for (i, p) in before.positions.iter().enumerate() {
            assert_eq!(mesh.uvs[i], before.uvs[i]);
            assert_eq!(ids[i], before_ids[i]);assert_eq!(weights[i], before_weights[i]);
            if !head(i) || (p[2]>=center.z+half.z*0.60 && p[1]>=center.y-half.y*0.40) {
                assert_eq!(mesh.positions[i], *p, "painted front/body shifted at {i}");
                protected_front+=usize::from(head(i));
            } else { rounded+=usize::from(mesh.positions[i]!=*p); }
        }
        assert!(protected_front>10 && rounded>10);
        assert!(mesh.positions.iter().flatten().all(|v| v.is_finite()));
        assert!(mesh.normals.iter().all(|n| (Vec3::from_array(*n).length()-1.0).abs()<1e-5));
        for w in weights { assert!(w.iter().all(|v| v.is_finite() && *v>=0.0));
            assert!((w.iter().sum::<f32>()-1.0).abs()<1e-5); }
    }
    #[test]
    #[ignore = "requires supplied male and female rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn adventure_face_keeps_body_uvs_skin_and_accessory_anchors() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for name in ["human-male", "human-female"] {
            let rig = PersonRig::parse(&std::fs::read(root.join(format!("people/kenney/{name}.glb"))).unwrap()).unwrap();
            let mut mesh = rig.mesh.clone();
            let mut ids = rig.ids.clone();
            let mut weights = rig.weights.clone();
            adventure_profile(&rig, &mut mesh, &mut ids, &mut weights);
            assert!(mesh.positions.len() > rig.mesh.positions.len());
            for i in 0..rig.mesh.positions.len() {
                assert_eq!(mesh.uvs[i], rig.mesh.uvs[i]);
                assert_eq!(ids[i], rig.ids[i]);
                assert_eq!(weights[i], rig.weights[i]);
                let head_weight: f32 = (0..4).filter(|&j| ids[i][j] as usize == rig.head_joint).map(|j| weights[i][j]).sum();
                if head_weight <= 0.5 { assert_eq!(mesh.positions[i], rig.mesh.positions[i]); }
                assert!(Vec3::from_array(mesh.positions[i]).distance(Vec3::from_array(rig.mesh.positions[i])) < 0.10);
            }
            for ((p, j), w) in mesh.positions.iter().zip(&ids).zip(&weights) {
                assert!(Vec3::from_array(*p).is_finite());
                assert!(j.iter().all(|&i| (i as usize) < rig.skin_nodes.len()));
                assert!(w.iter().all(|v| v.is_finite() && *v >= 0.0));
                assert!((w.iter().sum::<f32>() - 1.0).abs() < 1e-5);
            }
            assert!(mesh.normals.iter().all(|n| (Vec3::from_array(*n).length() - 1.0).abs() < 1e-5));
            assert_eq!(mesh.positions.len(), mesh.uvs.len());
            assert_eq!(mesh.positions.len(), ids.len());
        }
    }
    #[test]
    #[ignore = "requires supplied Kenney rig in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_bald_crowns_round_caps_preserve_faces_and_stay_in_animation_bounds() {
        use crate::new_bark_actors::character_meshes::CharacterKind::*;
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig = PersonRig::parse(&std::fs::read(root.join("people/kenney/human-male.glb")).unwrap()).unwrap();
        let mut base=rig.mesh.clone(); let mut base_ids=rig.ids.clone(); let mut base_weights=rig.weights.clone();
        smooth(&rig,&mut base,&mut base_ids,&mut base_weights);
        let is_head=|j:&[u16;4],w:&[f32;4]| (0..4).filter(|&i|j[i] as usize==rig.head_joint).map(|i|w[i]).sum::<f32>()>0.5;
        let ys=base.positions.iter().zip(&base_ids).zip(&base_weights)
            .filter(|((_,j),w)|is_head(j,w)).map(|((p,_),_)|p[1]).collect::<Vec<_>>();
        let lo=ys.iter().copied().fold(f32::INFINITY,f32::min);let hi=ys.iter().copied().fold(f32::NEG_INFINITY,f32::max);
        let threshold=(lo+hi)*0.5+(hi-lo)*0.175;
        for kind in [Blaine,Elder,Gramps,Pharmacist,PokefanM,Sage,Pryce,Kurt,KurtOutside,Scientist] {
            let mut mesh=base.clone();let mut ids=base_ids.clone();let mut weights=base_weights.clone();
            bald_crown(&rig,&mut mesh,&mut ids,&mut weights,crate::npc_appearance::look(kind));
            if kind==Scientist {
                assert_eq!(mesh.positions,base.positions);assert_eq!(mesh.normals,base.normals);
                assert_eq!(mesh.indices,base.indices);assert_eq!(mesh.uvs,base.uvs);
                assert_eq!(mesh.colors,base.colors);assert_eq!(ids,base_ids);assert_eq!(weights,base_weights);continue;
            }
            assert!(mesh.positions.len()>base.positions.len()+100);
            let mut moved=0;
            for i in 0..base.positions.len() {
                let d=Vec3::from_array(mesh.positions[i]).distance(Vec3::from_array(base.positions[i]));
                assert!(d<0.10,"excessive scalp shrink: {d}");moved+=usize::from(d>1e-5);
                if !is_head(&base_ids[i],&base_weights[i]) || base.positions[i][1]<=threshold {
                    assert_eq!(mesh.positions[i],base.positions[i]);assert_eq!(mesh.normals[i],base.normals[i]);
                }
                assert_eq!(mesh.uvs[i],base.uvs[i]);assert_eq!(mesh.colors[i],base.colors[i]);
                assert_eq!(ids[i],base_ids[i]);assert_eq!(weights[i],base_weights[i]);
            }
            assert!(moved>8,"cap corners were not rounded");
            let key=|p:[f32;3]|p.map(|v|(v*100_000.).round() as i32);
            let mut edges=std::collections::BTreeMap::<([i32;3],[i32;3]),(usize,i32)>::new();
            for face in mesh.indices.chunks_exact(3) {for e in 0..3 {
                let a=key(mesh.positions[face[e] as usize]);let b=key(mesh.positions[face[(e+1)%3] as usize]);
                let edge=edges.entry((a.min(b),a.max(b))).or_default();edge.0+=1;edge.1+=if a<b {1} else {-1};
            }}
            for ((a,b),count) in edges {
                if (a[1].min(b[1]) as f32)/100_000.>threshold-0.10 {
                    assert_eq!(count,(2,0),"{kind:?} unmatched crown edge {a:?} {b:?}");
                }
            }
            for face in mesh.indices.chunks_exact(3) {
                let p=face.iter().map(|&i|Vec3::from_array(mesh.positions[i as usize])).collect::<Vec<_>>();
                let n=(p[1]-p[0]).cross(p[2]-p[0]);let ns:Vec3=face.iter().map(|&i|Vec3::from_array(mesh.normals[i as usize])).sum();
                assert!(n.is_finite() && n.dot(ns)>=-1e-6,"reversed cap triangle");
                for edge in 0..3 {
                    let (a,b)=(face[edge] as usize,face[(edge+1)%3] as usize);
                    let distance=p[edge].distance(p[(edge+1)%3]);
                    if p[edge].y>threshold && p[(edge+1)%3].y>threshold && distance>1e-5 {
                        let delta=Vec3::from_array(mesh.normals[a]).distance(Vec3::from_array(mesh.normals[b]));
                        assert!(delta/distance<8.,"{kind:?} abrupt crown lighting gradient: {}",delta/distance);
                    }
                }
            }
            for ((n,j),w) in mesh.normals.iter().zip(&ids).zip(&weights) {
                assert!((Vec3::from_array(*n).length()-1.).abs()<1e-5);
                assert!(j.iter().all(|&i|(i as usize)<rig.skin_nodes.len()));
                assert!(w.iter().all(|&v|v.is_finite() && v>=0.));assert!((w.iter().sum::<f32>()-1.).abs()<1e-5);
            }
            fn world(i:usize,pose:&[Transform],parents:&[Option<usize>])->Mat4 {
                parents[i].map(|p|world(p,pose,parents)).unwrap_or(Mat4::IDENTITY)*pose[i].to_matrix()
            }
            for clip in ["Idle","Run"] {for step in 0..=60 {
                let mut pose=rig.nodes.clone();rig.sample(clip,rig.duration(clip)*step as f32/60.,&mut pose);
                let inverse=world(rig.mesh_node,&pose,&rig.parents).inverse();
                let skin=rig.skin_nodes.iter().zip(&rig.inverse_binds).map(|(&i,b)|inverse*world(i,&pose,&rig.parents)* *b).collect::<Vec<_>>();
                for ((p,j),w) in mesh.positions.iter().zip(&ids).zip(&weights) {
                    let p=Vec3::from_array(*p);let q:Vec3=(0..4).map(|i|skin[j[i] as usize].transform_point3(p)*w[i]).sum();
                    assert!(q.is_finite());assert!(q.cmpge(rig.bounds.0-Vec3::splat(0.02)).all() && q.cmple(rig.bounds.1+Vec3::splat(0.02)).all());
                }
            }}
        }
    }
    #[test]
    #[ignore = "requires local Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_heads_round_side_contours_without_moving_front_features_or_body() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for name in ["human-male", "human-female", "skater-male", "skater-female"] {
            let rig = PersonRig::parse(
                &std::fs::read(root.join(format!("people/kenney/{name}.glb"))).unwrap(),
            )
            .unwrap();
            let mut mesh = rig.mesh.clone();
            let mut ids = rig.ids.clone();
            let mut weights = rig.weights.clone();
            smooth(&rig, &mut mesh, &mut ids, &mut weights);
            assert!(
                mesh.positions.len() > rig.mesh.positions.len() + 20,
                "{name} missing refinement"
            );
            let is_head = |i: usize| {
                (0..4)
                    .filter(|&j| rig.ids[i][j] as usize == rig.head_joint)
                    .map(|j| rig.weights[i][j])
                    .sum::<f32>()
                    > 0.5
            };
            let max_z = rig
                .mesh
                .positions
                .iter()
                .enumerate()
                .filter(|(i, _)| is_head(*i))
                .map(|(_, p)| p[2])
                .fold(f32::NEG_INFINITY, f32::max);
            let min_z = rig
                .mesh
                .positions
                .iter()
                .enumerate()
                .filter(|(i, _)| is_head(*i))
                .map(|(_, p)| p[2])
                .fold(f32::INFINITY, f32::min);
            let front = (max_z + min_z) * 0.5 + (max_z - min_z) * 0.275;
            let min_y = rig
                .mesh
                .positions
                .iter()
                .enumerate()
                .filter(|(i, _)| is_head(*i))
                .map(|(_, p)| p[1])
                .fold(f32::INFINITY, f32::min);
            let max_y = rig
                .mesh
                .positions
                .iter()
                .enumerate()
                .filter(|(i, _)| is_head(*i))
                .map(|(_, p)| p[1])
                .fold(f32::NEG_INFINITY, f32::max);
            let feature_bottom = (max_y + min_y) * 0.5 - (max_y - min_y) * 0.21;
            let mut moved = 0;
            for i in 0..rig.mesh.positions.len() {
                let p = rig.mesh.positions[i];
                let distance = Vec3::from_array(p).distance(Vec3::from_array(mesh.positions[i]));
                assert!(distance < 0.06, "{name} overly rounded {distance}");
                moved += usize::from(distance > 1e-5);
                if !is_head(i) || (p[2] >= front && p[1] >= feature_bottom) {
                    assert_eq!(mesh.positions[i], p);
                    assert_eq!(mesh.normals[i], rig.mesh.normals[i]);
                }
                assert_eq!(mesh.uvs[i], rig.mesh.uvs[i]);
                assert_eq!(mesh.colors[i], rig.mesh.colors[i]);
                assert_eq!(ids[i], rig.ids[i]);
                assert_eq!(weights[i], rig.weights[i]);
            }
            assert!(moved > 3, "{name} no contour rounding");
            for face in mesh.indices.chunks_exact(3) {
                let p = [face[0], face[1], face[2]]
                    .map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let n = (p[1] - p[0]).cross(p[2] - p[0]);
                let normals: Vec3 = face
                    .iter()
                    .map(|&i| Vec3::from_array(mesh.normals[i as usize]))
                    .sum();
                assert!(
                    n.is_finite() && n.dot(normals) >= -1e-6,
                    "{name} reversed face"
                );
            }
            for (j, w) in ids.iter().zip(&weights) {
                assert!(j.iter().all(|&j| (j as usize) < rig.skin_nodes.len()));
                assert!(w.iter().all(|w| w.is_finite() && *w >= 0.));
                assert!((w.iter().sum::<f32>() - 1.).abs() < 1e-5);
            }
            assert!(mesh
                .normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 1e-5));
        }
    }
}

/// Bald crowns expose the source cap's broad polygon corners. Refine only
/// above the forehead, keeping the lower face and accessory support exact.
pub(crate) fn bald_crown(rig: &PersonRig, mesh: &mut SurfaceMeshData,
    ids: &mut Vec<[u16;4]>, weights: &mut Vec<[f32;4]>, look: crate::npc_appearance::Look) {
    if !matches!(look.hairstyle, "bald" | "sparse") || look.hat != "none" { return; }
    let head = |j: &[u16;4], w: &[f32;4]| (0..4)
        .filter(|&i| j[i] as usize == rig.head_joint).map(|i| w[i]).sum::<f32>() > 0.5;
    let mut lo = f32::INFINITY; let mut hi = f32::NEG_INFINITY;
    for ((p,j),w) in mesh.positions.iter().zip(ids.iter()).zip(weights.iter()) {
        if head(j,w) { lo=lo.min(p[1]); hi=hi.max(p[1]); }
    }
    if !lo.is_finite() || !hi.is_finite() { return; }
    let threshold = (lo+hi)*0.5 + (hi-lo)*0.175;
    conform_crown_edges(mesh,ids,weights,threshold);
    for _ in 0..2 {
        let selected = mesh.positions.iter().zip(ids.iter()).zip(weights.iter())
            .map(|((p,j),w)| head(j,w) && p[1]>threshold).collect::<Vec<_>>();
        crate::hand_surface::smooth_selection(mesh,ids,weights,&selected);
        conform_crown_edges(mesh,ids,weights,threshold);
    }
    // The selective cap's triangle areas vary strongly at its pinned ring.
    // A rounded cap normal field keeps lighting continuous across that topology.
    let mut bounds_lo=Vec3::splat(f32::INFINITY);
    let mut bounds_hi=Vec3::splat(f32::NEG_INFINITY);
    for ((p,j),w) in rig.mesh.positions.iter().zip(&rig.ids).zip(&rig.weights) {
        if head(j,w) { bounds_lo=bounds_lo.min(Vec3::from_array(*p)); bounds_hi=bounds_hi.max(Vec3::from_array(*p)); }
    }
    let center=(bounds_lo+bounds_hi)*0.5;
    let half=(bounds_hi-bounds_lo)*0.5;
    let cap_height=hi-threshold;
    for (((p,n),j),w) in mesh.positions.iter().zip(&mut mesh.normals).zip(ids.iter()).zip(weights.iter()) {
        if head(j,w) && p[1]>threshold {
            let target=Vec3::new((p[0]-center.x)/(half.x*half.x),
                (p[1]-threshold)/(cap_height*cap_height),
                (p[2]-center.z)/(half.z*half.z));
            if target.length_squared()>1e-12 {*n=target.normalize().to_array();}
        }
    }
}

/// Match the dyadic boundary vertices introduced by selective subdivision.
/// Neighbor faces retain their surface/UVs, but share the same geometric edges.
fn conform_crown_edges(mesh:&mut SurfaceMeshData,ids:&mut Vec<[u16;4]>,weights:&mut Vec<[f32;4]>,threshold:f32) {
    use std::collections::HashMap;
    let key=|p:[f32;3]|p.map(|x|(x*100_000.).round() as i32);
    for _ in 0..4 {
        let points:HashMap<_,_>=mesh.positions.iter().enumerate().filter(|(_,p)|p[1]>threshold-0.15)
            .map(|(i,&p)|(key(p),i)).collect();
        let mut counts=HashMap::new();
        for face in mesh.indices.chunks_exact(3) {for e in 0..3 {
            let a=key(mesh.positions[face[e] as usize]);let b=key(mesh.positions[face[(e+1)%3] as usize]);
            *counts.entry((a.min(b),a.max(b))).or_insert(0usize)+=1;
        }}
        let old=std::mem::take(&mut mesh.indices);let mut added=0;
        let mut cache=HashMap::<(u32,u32),u32>::new();
        for face in old.chunks_exact(3) {
            let mut mid=[None;3];
            for e in 0..3 {
                let (a,b)=(face[e],face[(e+1)%3]);let (ia,ib)=(a as usize,b as usize);
                let (ka,kb)=(key(mesh.positions[ia]),key(mesh.positions[ib]));
                let p=(Vec3::from_array(mesh.positions[ia])+Vec3::from_array(mesh.positions[ib]))*0.5;
                if p.y<=threshold-0.15 || counts[&(ka.min(kb),ka.max(kb))]!=1 || !points.contains_key(&key(p.to_array())) {continue;}
                let index=*cache.entry((a.min(b),a.max(b))).or_insert_with(|| {
                    let index=mesh.positions.len() as u32;
                    mesh.positions.push(p.to_array());
                    mesh.normals.push((Vec3::from_array(mesh.normals[ia])+Vec3::from_array(mesh.normals[ib])).normalize().to_array());
                    mesh.uvs.push(Vec2::from_array(mesh.uvs[ia]).lerp(Vec2::from_array(mesh.uvs[ib]),0.5).to_array());
                    mesh.colors.push(Vec4::from_array(mesh.colors[ia]).lerp(Vec4::from_array(mesh.colors[ib]),0.5).to_array());
                    let mut blend=std::collections::BTreeMap::<u16,f32>::new();
                    for i in [ia,ib] {for s in 0..4 {*blend.entry(ids[i][s]).or_default()+=weights[i][s]*0.5;}}
                    let mut blend=blend.into_iter().collect::<Vec<_>>();blend.sort_by(|a,b|b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));blend.truncate(4);
                    let total=blend.iter().map(|x|x.1).sum::<f32>();
                    ids.push(std::array::from_fn(|i|blend.get(i).map_or(0,|x|x.0)));
                    weights.push(std::array::from_fn(|i|blend.get(i).map_or(0.,|x|x.1/total)));index
                });mid[e]=Some(index);
            }
            match mid.iter().filter(|m|m.is_some()).count() {
                0=>mesh.indices.extend_from_slice(face),
                1=>{let e=mid.iter().position(Option::is_some).unwrap();let [a,b,c]=[face[e],face[(e+1)%3],face[(e+2)%3]];let m=mid[e].unwrap();mesh.indices.extend([a,m,c,m,b,c]);added+=1;},
                2=>{let e=(0..3).find(|&e|mid[e].is_some() && mid[(e+1)%3].is_some()).unwrap();let [a,b,c]=[face[e],face[(e+1)%3],face[(e+2)%3]];let (m,n)=(mid[e].unwrap(),mid[(e+1)%3].unwrap());mesh.indices.extend([a,m,c,m,n,c,m,b,n]);added+=1;},
                _=>{let [a,b,c]=[face[0],face[1],face[2]];let [m,n,o]=mid.map(Option::unwrap);mesh.indices.extend([a,m,o,m,b,n,o,n,c,m,n,o]);added+=1;}
            }
        }
        if added==0 {break;}
    }
}
