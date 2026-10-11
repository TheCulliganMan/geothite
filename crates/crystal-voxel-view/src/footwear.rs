//! Source shoe-island styling and gentle rounding with pinned soles/ankles.
use crate::{mesh::SurfaceMeshData, npc_appearance::Look};
use bevy::prelude::*;

fn shoe(p: [f32; 3], uv: [f32; 2]) -> bool {
    p[1] <= 0.27 && uv[0] >= 0.61 && uv[1] < 0.50
}
fn palette(look: Look) -> ([u8; 3], [u8; 3]) {
    if matches!(look.texture_key, "rocket" | "rocket_girl" | "surge") {
        // These uniforms call for dark boots, including the underside that
        // becomes prominent in Run. The generic pale sports sole looked bare.
        return ([43, 47, 53], [25, 27, 31]);
    }
    if matches!(look.texture_key, "cal" | "cooltrainer_m") {
        return (
            look.shirt.map(|v| (f32::from(v) * 0.65 + 15.).round() as u8),
            [65, 68, 73],
        );
    }
    if matches!(look.texture_key, "red" | "jasmine" | "karen" | "beauty" | "daisy" | "clair" | "sabrina" | "erika" | "bugsy" | "receptionist" | "whitney" | "misty" | "janine" | "koga" | "morty" | "falkner" | "lance" | "rival" | "blue" | "brock" | "elm" | "oak" | "scientist" | "blaine" | "pharmacist" | "elder" | "sage" | "pryce" | "kurt" | "kurt_outside" | "pokefan_m") {
        // These painted shoes already carry their intended palette.
        // A vest/dress tint obscured both the laces and upper color.
        return ([255, 255, 255], [163, 168, 168]);
    }
    if matches!(look.texture_key, "nurse" | "link_receptionist") {
        return ([232, 231, 219], [167, 172, 171]);
    }
    if matches!(look.costume, "suit" | "coat" | "sailor" | "robe" | "kimono") {
        // Charcoal uppers retain the rounded toe highlight at game scale;
        // near-black soles keep formal shoes grounded without flattening them.
        return ([65, 63, 69], [27, 28, 31]);
    }
    if look.costume == "vest" || look.texture_key == "outdoorsman" {
        return ([107, 77, 51], [40, 36, 33]);
    }
    if look.costume == "dress" || look.accessories.contains("skirt") {
        return (
            look.lower.map(|v| (f32::from(v) * 0.55).round() as u8),
            [38, 34, 38],
        );
    }
    if matches!(look.costume, "martial" | "swimsuit") {
        return ([225, 224, 213], [163, 168, 168]);
    }
    (
        look.shirt
            .map(|v| (f32::from(v) * 0.65 + 15.).round() as u8),
        [220, 218, 207],
    )
}

pub(crate) fn adapt(
    mesh: &mut SurfaceMeshData,
    ids: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
    look: Look,
) {
    if matches!(look.texture_key, "swimmer_guy" | "swimmer_girl") {
        bare_feet(mesh, ids, weights);
        return;
    }
    let selected: Vec<_> = mesh
        .positions
        .iter()
        .zip(&mesh.uvs)
        .map(|(&p, &uv)| shoe(p, uv) && p[1] > 0.015 && p[1] < 0.235)
        .collect();
    crate::hand_surface::smooth_selection(mesh, ids, weights, &selected);
    let (upper, sole) = palette(look);
    let linear = |rgb: [u8; 3]| rgb.map(|v| Color::srgb_u8(v, v, v).to_linear().red);
    let upper = linear(upper);
    let sole = linear(sole);
    for ((&p, &uv), color) in mesh.positions.iter().zip(&mesh.uvs).zip(&mut mesh.colors) {
        if !shoe(p, uv) {
            continue;
        }
        let blend = ((p[1] - 0.055) / 0.045).clamp(0., 1.);
        let blend = blend * blend * (3. - 2. * blend);
        for axis in 0..3 {
            color[axis] *= sole[axis] + (upper[axis] - sole[axis]) * blend;
        }
        if look.texture_key == "receptionist" {
            // Source heel UVs reach the shoe strip's pale surround. Keep
            // that rear surface charcoal without obscuring the front laces.
            let front = ((p[2] + 0.01) / 0.10).clamp(0., 1.);
            let front = front * front * (3. - 2. * front);
            for channel in &mut color[..3] {
                *channel *= 0.08 + 0.92 * front;
            }
        }
        if matches!(look.texture_key, "koga" | "janine" | "morty" | "falkner" | "lance" | "rival" | "blue" | "brock" | "elm" | "oak" | "scientist" | "blaine" | "pharmacist" | "elder" | "sage" | "pryce" | "kurt" | "kurt_outside" | "pokefan_m") {
            // These paintings have dark uppers, but the original
            // heel UVs sample their grey gutter. Retain the grey sole and
            // front panels while bringing the upper heel into that palette.
            let front = ((p[2] + 0.01) / 0.10).clamp(0., 1.);
            let front = front * front * (3. - 2. * front);
            for channel in &mut color[..3] {
                *channel *= 1.0 - 0.75 * (1.0 - front) * blend;
            }
        }
    }
}

/// Reshape the connected foot surface, retaining the ankle ring and floor.
/// Toe contours belong to that surface, so no separate toe pieces can float
/// away from the instep when the production skeleton bends the ankle.
fn bare_feet(mesh: &mut SurfaceMeshData, ids: &mut Vec<[u16;4]>, weights: &mut Vec<[f32;4]>) {
    for _ in 0..2 {
        let selected: Vec<_> = mesh.positions.iter().map(|p| p[1]>0.012 && p[1]<0.235).collect();
        crate::hand_surface::smooth_selection(mesh, ids, weights, &selected);
    }
    let mut ankle_points = [std::collections::BTreeSet::new(), std::collections::BTreeSet::new()];
    for &p in &mesh.positions {
        if (0.235..0.27).contains(&p[1]) {
            ankle_points[usize::from(p[0]>0.0)].insert(p.map(|v|(v*100_000.0).round() as i32));
        }
    }
    let centers = ankle_points.map(|points| {
        assert!(!points.is_empty(), "supplied foot has no ankle ring");
        points.iter().map(|p|p[0] as f32/100_000.0).sum::<f32>()/points.len() as f32
    });
    for (p, uv) in mesh.positions.iter_mut().zip(&mut mesh.uvs) {
        if p[1]>=0.27 {continue;}
        let side = usize::from(p[0]>0.0);
        let t=((p[1]-0.14)/0.095).clamp(0.0,1.0);
        let fade=1.0-t*t*(3.0-2.0*t);
        let forward=((p[2]+0.05)/0.28).clamp(0.0,1.0);
        let width=0.72+0.12*forward;
        if fade>0.0 {
            p[0]=centers[side]+(p[0]-centers[side])*(1.0-(1.0-width)*fade);
            if p[1]>0.012 {p[1]*=1.0-(0.22+0.20*forward)*fade;}
        }
        // The inside toe is longest; the outer toe tapers back. Shallow
        // valleys delineate five toes within one continuous rounded surface.
        let across=((p[0]-centers[side])*p[0].signum()/0.26+0.5).clamp(0.0,1.0);
        let toe=((p[2]-0.14)/0.12).clamp(0.0,1.0);
        let lobes=(across*std::f32::consts::PI*5.0).sin().abs();
        if fade>0.0 {p[2]-=fade*toe*(0.018+across*0.035+(1.0-lobes)*0.012);}
        // The same inspected plain skin patch as the exposed lower legs.
        *uv=[0.90+(p[0]-centers[side])*0.025,0.31+p[2]*0.025];
    }
    // Recompute normals after changing the volume. Weld coincident UV seams
    // for shading, keeping the individual skinning influences untouched.
    let key=|p:[f32;3]|p.map(|v|(v*100_000.0).round() as i32);
    let mut normals=std::collections::HashMap::<[i32;3],Vec3>::new();
    for face in mesh.indices.chunks_exact(3) {
        let p=[face[0],face[1],face[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
        let normal=(p[1]-p[0]).cross(p[2]-p[0]);
        for &i in face {
            if mesh.positions[i as usize][1]<0.27 {
                *normals.entry(key(mesh.positions[i as usize])).or_default()+=normal;
            }
        }
    }
    for i in 0..mesh.positions.len() {
        if mesh.positions[i][1]<0.27 {
            if let Some(n)=normals.get(&key(mesh.positions[i])) {
                if n.length_squared()>1e-16 {mesh.normals[i]=n.normalize().to_array();}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{new_bark_actors::character_meshes::CharacterKind, open_people::PersonRig};
    #[test]
    #[ignore = "requires supplied swimmer rigs and paintings"]
    fn supplied_swimmer_feet_have_skin_round_contours_and_pinned_ankles() {
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for kind in [CharacterKind::SwimmerGuy,CharacterKind::SwimmerGirl] {
            let look=crate::npc_appearance::look(kind);
            let rig=PersonRig::parse(&std::fs::read(root.join(format!("people/kenney/{}.glb",look.base))).unwrap()).unwrap();
            let painting=crate::npc_textures::decode(&std::fs::read(root.join(format!("people/npc-textures/{}.png",look.texture_key))).unwrap()).unwrap();
            let mut mesh=rig.mesh.clone();let mut ids=rig.ids.clone();let mut weights=rig.weights.clone();
            adapt(&mut mesh,&mut ids,&mut weights,look);
            let mut feet=[0;2];let mut changed=0;
            for &i in &mesh.indices {
                let i=i as usize;let p=mesh.positions[i];
                assert!(p.iter().all(|v|v.is_finite()));
                assert!((weights[i].iter().sum::<f32>()-1.0).abs()<0.001);
                if p[1]>=0.27 {continue;}
                feet[usize::from(p[0]>0.0)]+=1;
                let [u,v]=mesh.uvs[i];
                let x=(u*painting.size.0 as f32) as u32;let y=(v*painting.size.1 as f32) as u32;
                let rgb=&painting.rgba[((y*painting.size.0+x)*4) as usize..][..3];
                assert!(rgb[0]>160 && rgb[0]>rgb[1] && rgb[1]>rgb[2],"foot samples shoe or gutter: {rgb:?}");
                assert!(p[1]>=rig.floor-1e-5,"foot through floor: {p:?}");
                assert!((Vec3::from_array(mesh.normals[i]).length()-1.0).abs()<0.001);
            }
            for i in 0..rig.mesh.positions.len() {
                if rig.mesh.positions[i][1]>=0.235 {
                    assert_eq!(mesh.positions[i],rig.mesh.positions[i],"ankle/other body moved");
                } else if mesh.positions[i]!=rig.mesh.positions[i] {changed+=1;}
                if rig.mesh.positions[i][1]>=0.27 {assert_eq!(mesh.uvs[i],rig.mesh.uvs[i]);}
                if rig.mesh.positions[i][1]<0.012 {assert_eq!(mesh.positions[i][1],rig.mesh.positions[i][1]);}
                assert_eq!(ids[i],rig.ids[i]);assert_eq!(weights[i],rig.weights[i]);
            }
            assert!(feet.iter().all(|&n|n>1000),"missing rounded feet {feet:?}");
            assert!(changed>100,"shoe volume was only recolored: {changed}");
        }
    }
    #[test]
    #[ignore = "requires locally supplied Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_footwear_rounds_shoes_and_preserves_soles_ankles_and_other_islands() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for name in ["human-male", "human-female", "skater-male", "skater-female"] {
            let rig = PersonRig::parse(
                &std::fs::read(root.join(format!("people/kenney/{name}.glb"))).unwrap(),
            )
            .unwrap();
            let mut mesh = rig.mesh.clone();
            let mut ids = rig.ids.clone();
            let mut weights = rig.weights.clone();
            adapt(
                &mut mesh,
                &mut ids,
                &mut weights,
                crate::npc_appearance::look(CharacterKind::Officer),
            );
            assert!(
                mesh.positions.len() > rig.mesh.positions.len() + 80,
                "{name} missing shoe rounding"
            );
            let mut moved = 0;
            let mut colored = 0;
            for i in 0..rig.mesh.positions.len() {
                let p = rig.mesh.positions[i];
                let dist = Vec3::from_array(p).distance(Vec3::from_array(mesh.positions[i]));
                assert!(dist < 0.06);
                if !shoe(p, rig.mesh.uvs[i]) {
                    assert_eq!(mesh.positions[i], p);
                    assert_eq!(mesh.colors[i], rig.mesh.colors[i]);
                    assert_eq!(mesh.normals[i], rig.mesh.normals[i]);
                } else {
                    if p[1] <= 0.015 || p[1] >= 0.235 {
                        assert_eq!(mesh.positions[i], p);
                    }
                    moved += usize::from(dist > 1e-5);
                    colored += usize::from(mesh.colors[i] != rig.mesh.colors[i]);
                }
                assert_eq!(ids[i], rig.ids[i]);
                assert_eq!(weights[i], rig.weights[i]);
                assert_eq!(mesh.uvs[i], rig.mesh.uvs[i]);
            }
            assert!(
                moved > 15 && colored > 80,
                "{name} did not style/round shoes"
            );
            for face in mesh.indices.chunks_exact(3) {
                let p = [face[0], face[1], face[2]]
                    .map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let n = (p[1] - p[0]).cross(p[2] - p[0]);
                let normals: Vec3 = face
                    .iter()
                    .map(|&i| Vec3::from_array(mesh.normals[i as usize]))
                    .sum();
                assert!(n.is_finite() && n.dot(normals) >= -1e-6);
            }
            for (j, w) in ids.iter().zip(&weights) {
                assert!(j.iter().all(|&id| (id as usize) < rig.skin_nodes.len()));
                assert!(w.iter().all(|w| w.is_finite() && *w >= 0.));
                assert!((w.iter().sum::<f32>() - 1.).abs() < 1e-5);
            }
            assert!(
                mesh.normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 1e-5)
            );
            for step in 0..61 {
                let mut pose = rig.nodes.clone();
                rig.sample("Run", rig.duration("Run") * step as f32 / 60., &mut pose);
                fn world(i: usize, pose: &[Transform], parents: &[Option<usize>]) -> Mat4 {
                    parents[i]
                        .map(|p| world(p, pose, parents))
                        .unwrap_or(Mat4::IDENTITY)
                        * pose[i].to_matrix()
                }
                let inverse = world(rig.mesh_node, &pose, &rig.parents).inverse();
                let skin: Vec<_> = rig
                    .skin_nodes
                    .iter()
                    .zip(&rig.inverse_binds)
                    .map(|(&node, bind)| inverse * world(node, &pose, &rig.parents) * *bind)
                    .collect();
                for ((&p, &j), &w) in mesh.positions.iter().zip(&ids).zip(&weights) {
                    let p = (0..4)
                        .map(|i| skin[j[i] as usize].transform_point3(Vec3::from_array(p)) * w[i])
                        .sum::<Vec3>();
                    assert!(
                        p.is_finite()
                            && p.cmpge(rig.bounds.0 - Vec3::splat(0.002)).all()
                            && p.cmple(rig.bounds.1 + Vec3::splat(0.002)).all(),
                        "{name} footwear escapes animation envelope"
                    );
                }
            }
        }
    }
}
