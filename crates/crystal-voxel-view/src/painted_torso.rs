//! Place front clothing paintings on the source rig's separate front island.
use crate::{mesh::SurfaceMeshData, npc_appearance::Look};
pub(crate) mod cuffs;

pub(crate) fn has_short_sleeves(look: Look) -> bool {
    matches!(look.texture_key, "red" | "misty" | "brock" | "pokefan_m" | "nurse")
}

#[derive(Clone, Copy)]
struct SleevePaint {
    u: f32,
    z_scale: f32,
    v: f32,
    x_origin: f32,
    x_scale: f32,
    v_limits: Option<(f32, f32)>,
}
impl SleevePaint {
    fn for_surface(look: Look, cloth: bool) -> Self {
        if cloth {
            if matches!(look.texture_key, "misty" | "pokefan_m") {
                Self { u: 0.075, z_scale: 0.04,
                    v: if look.texture_key == "pokefan_m" {0.68} else {0.77},
                    x_origin: 0.4, x_scale: 0.10, v_limits: None }
            } else {
                Self { u: 0.3125, z_scale: 0.04, v: 0.70,
                    x_origin: 0.4, x_scale: 0.15, v_limits: None }
            }
        } else if matches!(look.texture_key, "brock" | "pokefan_m") {
            Self { u: 0.87, z_scale: 0.06, v: 0.69,
                x_origin: 0.8, x_scale: 0.08, v_limits: Some((0.67,0.733)) }
        } else {
            Self { u: 0.87, z_scale: 0.06, v: 0.31,
                x_origin: 0.8, x_scale: 0.12, v_limits: None }
        }
    }
    fn sample(self, p: [f32;3]) -> [f32;2] {
        let v = self.v + (p[0].abs()-self.x_origin)*self.x_scale;
        [self.u+p[2]*self.z_scale,
            self.v_limits.map_or(v, |(low,high)| v.clamp(low,high))]
    }
    fn position(self, uv: [f32;2]) -> [f32;3] {
        // The cuff is inside the unclamped skin patch. Recover the original
        // painting coordinates, including refinement before the body rounded.
        [(uv[1]-self.v)/self.x_scale+self.x_origin, 0.,
            (uv[0]-self.u)/self.z_scale]
    }
}

/// These cloth rectangles have gray separator padding inside the source
/// shirt island. Keep complete cloth faces within the painted fabric, leaving
/// adjacent neck, hands and trousers on their own UV islands.
pub(crate) fn avoid_shirt_gutter(mesh: &mut SurfaceMeshData, look: Look,
    joints: &mut Vec<[u16; 4]>, weights: &mut Vec<[f32; 4]>) {
    if !matches!(look.texture_key, "will" | "surge" | "gentleman" | "rocket"
        | "fishing_guru" | "gameboy_kid" | "biker" | "cooltrainer_m"
        | "bill" | "cal" | "gym_guide" | "standing_youngster"
        | "clerk" | "outdoorsman" | "unused_guy" | "bug_catcher" | "rocker") { return; }
    for face in mesh.indices.chunks_exact_mut(3) {
        let ids = [face[0] as usize, face[1] as usize, face[2] as usize];
        if !ids.iter().all(|&i| (1.34..2.57).contains(&mesh.positions[i][1])
            && mesh.uvs[i][0] < 0.625 && mesh.uvs[i][1] > 0.46) { continue; }
        if !ids.iter().any(|&i| mesh.uvs[i][0] > 0.595 || mesh.uvs[i][1] < 0.51) {
            continue;
        }
        for (index, &i) in face.iter_mut().zip(&ids) {
            *index = mesh.positions.len() as u32;
            mesh.positions.push(mesh.positions[i]);
            mesh.normals.push(mesh.normals[i]);
            mesh.colors.push(mesh.colors[i]);
            mesh.uvs.push([mesh.uvs[i][0].clamp(0.005, 0.595),
                mesh.uvs[i][1].clamp(0.51, 0.99)]);
            joints.push(joints[i]); weights.push(weights[i]);
        }
    }
}

pub(crate) fn map_front(mesh: &mut SurfaceMeshData, look: Look,
    joints: &mut Vec<[u16; 4]>, weights: &mut Vec<[f32; 4]>) {
    let width = match look.texture_key {
        "will" => 0.25,
        "chris" | "super_nerd" | "gramps" | "youngster" | "kris" | "red" | "mom" | "bugsy" | "koga" | "morty" | "falkner" | "lance" | "rival" | "blue" | "brock" | "elm" | "oak" | "scientist" | "blaine" | "pharmacist" | "elder" | "sage" | "pryce" | "kurt" | "kurt_outside" | "pokefan_m" => 0.90,
        "lass" | "nurse" | "cooltrainer_f" | "granny" | "jasmine" | "karen" | "beauty" | "daisy" | "clair" | "sabrina" | "erika" | "receptionist" | "whitney" | "misty" | "janine" | "kimono_girl" => 0.56,
        _ => return,
    };
    map_back(mesh, look, joints, weights);
    lab_coat_collar_edges(mesh, look, joints, weights);
    // The source front shirt starts at v=.747, whereas painted collars start
    // at v=.475. Neck corners at the same positions use a separate skin UV.
    // Split whole faces to leave that skin island and the side seams intact.
    for face in mesh.indices.chunks_exact_mut(3) {
        let ids = [face[0] as usize, face[1] as usize, face[2] as usize];
        let front = ids.iter().map(|&i| mesh.positions[i][2]).sum::<f32>() / 3.0;
        let height = ids.iter().map(|&i| mesh.positions[i][1]).sum::<f32>() / 3.0;
        if front <= 0.03 || !(1.25..2.55).contains(&height)
            || !ids.iter().all(|&i| mesh.positions[i][0].abs() < 0.4
                && mesh.uvs[i][0] < 0.5 && mesh.uvs[i][1] >= 0.74) {
            continue;
        }
        for (index, &i) in face.iter_mut().zip(&ids) {
            let p = mesh.positions[i];
            *index = mesh.positions.len() as u32;
            mesh.positions.push(p);
            mesh.normals.push(mesh.normals[i]);
            mesh.colors.push(mesh.colors[i]);
            // These contrasting lower islands start at u=.605; keep the
            // outer front shirt corners inside the adjacent fabric gutter.
            let (left, right) = if matches!(look.texture_key, "morty" | "falkner" | "lance" | "rival" | "blue" | "brock" | "elm" | "oak" | "scientist" | "blaine" | "pharmacist" | "elder" | "sage" | "pryce" | "kurt" | "kurt_outside" | "pokefan_m") { (0.01, 0.595) }
                else if width > 0.8 { (0.01, 0.615) } else { (0.13, 0.49) };
            let u = if look.texture_key == "will" {
                (0.3074 + p[0] * width).clamp(0.235, 0.38)
            } else {
                (0.3125 + p[0] * width).clamp(left, right)
            };
            let mut v = (0.475 + (2.52 - p[1]) * 0.45).clamp(0.475, 0.965);
            if look.texture_key == "will" {
                // The candidate's measured front collar starts at v=.7097.
                // Its separate rear island stays on uninterrupted red cloth.
                v = (0.7097 + (2.52 - p[1]) * 0.235).clamp(0.7097, 0.985);
            }
            if matches!(look.texture_key, "clair" | "sabrina") {
                // These raised collars extend above the shoulder fabric in
                // the painting. Keep the source shoulder ring below the skin
                // strip while retaining the centered high neckline.
                let shoulder = ((p[0].abs() - 0.10) / 0.06).clamp(0.0, 1.0);
                v = v.max(0.475 + 0.035 * shoulder * shoulder * (3.0 - 2.0 * shoulder));
            }
            mesh.uvs.push([u, v]);
            joints.push(joints[i]);
            weights.push(weights[i]);
        }
    }
    kimono_shoulders(mesh, look, joints, weights);
}

/// Shoulder and underarm faces use a separate source island that crosses the
/// blossom artwork. Include the sleeve root below the shoulder cap: remapping
/// only its top left a second, torn-looking flower in the rear armpit. Keep
/// complete cloth faces on plain fabric; neck and central front UVs stay separate.
fn kimono_shoulders(mesh: &mut SurfaceMeshData, look: Look,
    joints: &mut Vec<[u16; 4]>, weights: &mut Vec<[f32; 4]>) {
    if look.texture_key != "kimono_girl" { return; }
    for face in mesh.indices.chunks_exact_mut(3) {
        let ids=[face[0] as usize,face[1] as usize,face[2] as usize];
        let outer=ids.iter().map(|&i|mesh.positions[i][0].abs()).sum::<f32>()/3.0;
        let rear=ids.iter().map(|&i|mesh.positions[i][2]).sum::<f32>()/3.0;
        let cap = (outer > 0.18 || rear < 0.03)
            && ids.iter().all(|&i| mesh.positions[i][1] > 2.30);
        let root = ids.iter().all(|&i| mesh.positions[i][0].abs() > 0.30)
            && ids.iter().any(|&i| mesh.positions[i][0].abs() < 0.55)
            && ids.iter().any(|&i| mesh.positions[i][0].abs() > 0.395);
        if !(cap || root) || !ids.iter().all(|&i|
            mesh.positions[i][1] > 1.90 && mesh.uvs[i][0] < 0.625
            && mesh.uvs[i][1] >= 0.475) { continue; }
        for (index,&i) in face.iter_mut().zip(&ids) {
            let p=mesh.positions[i];*index=mesh.positions.len() as u32;
            mesh.positions.push(p);mesh.normals.push(mesh.normals[i]);mesh.colors.push(mesh.colors[i]);
            mesh.uvs.push([0.075+p[2]*0.01,0.64+(2.60-p[1])*0.10]);
            joints.push(joints[i]);weights.push(weights[i]);
        }
    }
}

/// The source side collar crosses the front tie island below v=.74.
/// Keep those whole faces on plain coat fabric, leaving the neck skin intact.
fn lab_coat_collar_edges(mesh: &mut SurfaceMeshData, look: Look,
    joints: &mut Vec<[u16; 4]>, weights: &mut Vec<[f32; 4]>) {
    if !matches!(look.texture_key, "elm" | "oak" | "scientist" | "blaine" | "pharmacist" | "elder" | "sage" | "pryce" | "kurt" | "kurt_outside" | "pokefan_m" | "kimono_girl") { return; }
    for face in mesh.indices.chunks_exact_mut(3) {
        let ids = [face[0] as usize, face[1] as usize, face[2] as usize];
        if !ids.iter().all(|&i| (2.40..2.56).contains(&mesh.positions[i][1])
            && mesh.positions[i][0].abs() < 0.4 && mesh.uvs[i][0] < 0.625
            && mesh.uvs[i][1] > 0.60)
            || !ids.iter().any(|&i| mesh.uvs[i][1] < 0.74) { continue; }
        for (index, &i) in face.iter_mut().zip(&ids) {
            let p = mesh.positions[i];
            *index = mesh.positions.len() as u32;
            mesh.positions.push(p); mesh.normals.push(mesh.normals[i]);
            mesh.colors.push(mesh.colors[i]);
            mesh.uvs.push([0.075 + p[0]*0.01, 0.64 + (2.55-p[1])*0.15]);
            joints.push(joints[i]); weights.push(weights[i]);
        }
    }
}

fn map_back(mesh: &mut SurfaceMeshData, look: Look,
    joints: &mut Vec<[u16; 4]>, weights: &mut Vec<[f32; 4]>) {
    let patch = match look.texture_key {
        "youngster" | "bugsy" => [0.20, 0.64, 0.025, 0.15],
        "kris" | "red" | "mom" | "jasmine" | "karen" | "beauty" | "daisy" | "clair" => [0.16, 0.65, 0.025, 0.15],
        // Sabrina's gold side seam crosses the usual rear fabric patch.
        "sabrina" => [0.24, 0.65, 0.01, 0.15],
        "erika" => [0.24, 0.66, 0.01, 0.15],
        // Stay outside the Receptionist's front lapel and pocket artwork.
        "receptionist" | "kimono_girl" => [0.075, 0.62, 0.008, 0.15],
        "whitney" => [0.075, 0.78, 0.008, 0.12],
        "misty" => [0.075, 0.77, 0.008, 0.12],
        "janine" | "koga" | "morty" | "falkner" | "lance" | "rival" | "blue" | "brock" | "elm" | "oak" | "scientist" | "blaine" | "pharmacist" | "elder" | "sage" | "pryce" | "kurt" | "kurt_outside" | "pokefan_m" => [0.075, 0.64, 0.008, 0.15],
        "granny" => [0.16, 0.55, 0.010, 0.10],
        _ => return,
    };
    // These two paintings also put front details on the source rear island:
    // a neckline on Youngster's lower back and Granny's knot behind her neck.
    for face in mesh.indices.chunks_exact_mut(3) {
        let ids = [face[0] as usize, face[1] as usize, face[2] as usize];
        let rear = ids.iter().map(|&i| mesh.positions[i][2]).sum::<f32>() / 3.0;
        let height = ids.iter().map(|&i| mesh.positions[i][1]).sum::<f32>() / 3.0;
        if rear >= 0.03 || !(1.25..2.55).contains(&height)
            || !ids.iter().all(|&i| mesh.positions[i][0].abs() < 0.4
                && mesh.positions[i][2] <= 0.1 && mesh.uvs[i][0] < 0.5
                && mesh.uvs[i][1] > 0.47) { continue; }
        for (index, &i) in face.iter_mut().zip(&ids) {
            let p = mesh.positions[i];
            *index = mesh.positions.len() as u32;
            mesh.positions.push(p);
            mesh.normals.push(mesh.normals[i]);
            mesh.colors.push(mesh.colors[i]);
            mesh.uvs.push([patch[0] + p[0] * patch[2], patch[1] + (2.55 - p[1]) * patch[3]]);
            joints.push(joints[i]);
            weights.push(weights[i]);
        }
    }
}

/// Keep painted coat/robe front details out of the source arm island.
pub(crate) fn long_sleeves(mesh: &mut SurfaceMeshData,
    joints: &mut Vec<[u16; 4]>, weights: &mut Vec<[f32; 4]>, look: Look) {
    if !matches!(look.texture_key, "elm" | "oak" | "scientist" | "blaine" | "pharmacist" | "elder" | "sage" | "pryce" | "kurt" | "kurt_outside" | "kimono_girl") { return; }
    for face in mesh.indices.chunks_exact_mut(3) {
        let ids = [face[0] as usize, face[1] as usize, face[2] as usize];
        if !ids.iter().all(|&i| mesh.positions[i][0].abs() > 0.4
            && (2.0..2.55).contains(&mesh.positions[i][1])
            && mesh.uvs[i][0] < 0.625 && mesh.uvs[i][1] > 0.60) { continue; }
        for (index, &i) in face.iter_mut().zip(&ids) {
            let p = mesh.positions[i];
            *index = mesh.positions.len() as u32;
            mesh.positions.push(p);
            mesh.normals.push(mesh.normals[i]);
            mesh.colors.push(mesh.colors[i]);
            mesh.uvs.push([0.075 + p[2]*0.04, 0.68 + (p[0].abs()-0.4)*0.10]);
            joints.push(joints[i]); weights.push(weights[i]);
        }
    }
}

/// Split whole faces at the source arm ring to expose the forearms.
pub(crate) fn short_sleeves(mesh: &mut SurfaceMeshData,
    joints: &mut Vec<[u16; 4]>, weights: &mut Vec<[f32; 4]>, look: Look) {
    for face in mesh.indices.chunks_exact_mut(3) {
        let ids = [face[0] as usize, face[1] as usize, face[2] as usize];
        if !ids.iter().all(|&i| mesh.positions[i][0].abs() > 0.4
            && (2.0..2.55).contains(&mesh.positions[i][1])
            && mesh.uvs[i][0] < 0.625 && mesh.uvs[i][1] > 0.60) { continue; }
        // End at a complete source arm ring. Mean-position classification
        // split the two triangles of a quad and left a sawtooth sleeve hem.
        let sleeve = ids.iter().all(|&i| mesh.positions[i][0].abs() < 0.74);
        for (index, &i) in face.iter_mut().zip(&ids) {
            let p = mesh.positions[i];
            *index = mesh.positions.len() as u32;
            mesh.positions.push(p);
            mesh.normals.push(mesh.normals[i]);
            mesh.colors.push(mesh.colors[i]);
            mesh.uvs.push(SleevePaint::for_surface(look,sleeve).sample(p));
            joints.push(joints[i]);
            weights.push(weights[i]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{new_bark_actors::character_meshes::CharacterKind, open_people::PersonRig};

    #[test]
    #[ignore = "requires supplied rigs and shirt-gutter paintings"]
    fn supplied_shirt_gutters_sample_cloth_and_preserve_other_islands() {
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for (_, kind) in crate::new_bark_actors::character_meshes::SOURCE_KINDS {
            let look=crate::npc_appearance::look(kind);
            let rig=PersonRig::parse(&std::fs::read(root.join(format!(
                "people/kenney/{}.glb",look.base))).unwrap()).unwrap();
            let mut mesh=rig.mesh.clone();let mut ids=rig.ids.clone();let mut weights=rig.weights.clone();
            avoid_shirt_gutter(&mut mesh,look,&mut ids,&mut weights);
            if !matches!(kind, CharacterKind::Will | CharacterKind::Surge
                | CharacterKind::Gentleman | CharacterKind::Rocket
                | CharacterKind::FishingGuru | CharacterKind::GameboyKid | CharacterKind::Biker
                | CharacterKind::CooltrainerM | CharacterKind::Bill | CharacterKind::Cal
                | CharacterKind::GymGuide | CharacterKind::StandingYoungster
                | CharacterKind::Clerk | CharacterKind::Outdoorsman | CharacterKind::UnusedGuy
                | CharacterKind::BugCatcher | CharacterKind::Rocker) {
                assert_eq!(mesh.indices,rig.mesh.indices);
                assert_eq!(mesh.uvs,rig.mesh.uvs);
                assert_eq!(mesh.positions,rig.mesh.positions);
                assert_eq!(ids,rig.ids);assert_eq!(weights,rig.weights);
                continue;
            }
            let painting=crate::npc_textures::decode(&std::fs::read(root.join(format!(
                "people/npc-textures/{}.png",look.texture_key))).unwrap()).unwrap();
            let mut changed=0;
            for (source, updated) in rig.mesh.indices.chunks_exact(3).zip(mesh.indices.chunks_exact(3)) {
                for (&a,&b) in source.iter().zip(updated) {
                    let (a,b)=(a as usize,b as usize);
                    assert_eq!(mesh.positions[b],rig.mesh.positions[a]);
                    assert_eq!(mesh.normals[b],rig.mesh.normals[a]);
                    assert_eq!(mesh.colors[b],rig.mesh.colors[a]);
                    assert_eq!(ids[b],rig.ids[a]);assert_eq!(weights[b],rig.weights[a]);
                    if a==b {assert_eq!(mesh.uvs[b],rig.mesh.uvs[a]);}
                }
                if source==updated {continue;}
                changed+=1;
                // Check the interior as well as corners: UV interpolation
                // must not cross the separator, skin, or footwear artwork.
                for a in 0..=8 { for b in 0..=8-a {
                    let bary=[a as f32/8.,b as f32/8.,(8-a-b) as f32/8.];
                    let uv: [f32;2]=std::array::from_fn(|axis| (0..3)
                        .map(|i|mesh.uvs[updated[i] as usize][axis]*bary[i]).sum());
                    let x=(uv[0]*painting.size.0 as f32) as u32;
                    let y=(uv[1]*painting.size.1 as f32) as u32;
                    let rgb=&painting.rgba[((y*painting.size.0+x)*4) as usize..][..3];
                    let cloth=if matches!(kind,CharacterKind::Will | CharacterKind::Bill | CharacterKind::Cal) {
                        rgb[0] as f32 > rgb[1] as f32*1.3 && rgb[0] as f32 > rgb[2] as f32*1.3
                    } else if matches!(kind, CharacterKind::Surge | CharacterKind::Clerk | CharacterKind::Outdoorsman | CharacterKind::Rocker) {
                        rgb[1] as f32 > rgb[0] as f32*1.12 && rgb[1] as f32 > rgb[2] as f32*1.3
                    } else if kind==CharacterKind::UnusedGuy {
                        rgb[0] as f32 > rgb[1] as f32*1.1 && rgb[1] as f32 > rgb[2] as f32*1.5
                    } else {
                        rgb[2] as f32 > rgb[1] as f32*1.2 && rgb[1] as f32 > rgb[0] as f32*1.1
                    };
                    assert!(cloth,"{} shirt sampled non-cloth {rgb:?} at {uv:?}",look.texture_key);
                }}
            }
            assert!(changed>6,"{} gutter correction absent",look.texture_key);
            assert_eq!(mesh.positions.len(),ids.len());assert_eq!(ids.len(),weights.len());
        }
    }

    #[test]
    #[ignore = "requires supplied Kimono Girl painting and female rig"]
    fn supplied_kimono_girl_plain_regions_preserve_skin_and_rig() {
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig=PersonRig::parse(&std::fs::read(root.join("people/kenney/human-female.glb")).unwrap()).unwrap();
        let look=crate::npc_appearance::look(CharacterKind::KimonoGirl);
        let painting=crate::npc_textures::decode(&std::fs::read(root.join("people/npc-textures/kimono_girl.png")).unwrap()).unwrap();
        let operations: [fn(&mut SurfaceMeshData,&mut Vec<[u16;4]>,&mut Vec<[f32;4]>,Look);4] = [long_sleeves,
            |m,j,w,l| lab_coat_collar_edges(m,l,j,w), |m,j,w,l| map_back(m,l,j,w),
            |m,j,w,l| kimono_shoulders(m,l,j,w)];
        for operation in operations {
            let mut mesh=rig.mesh.clone();let mut joints=rig.ids.clone();let mut weights=rig.weights.clone();
            operation(&mut mesh,&mut joints,&mut weights,look);
            let mut changed=[0;2];
            for (&a,&b) in rig.mesh.indices.iter().zip(&mesh.indices) {
                let (a,b)=(a as usize,b as usize);
                assert_eq!(mesh.positions[b],rig.mesh.positions[a]);assert_eq!(mesh.normals[b],rig.mesh.normals[a]);
                assert_eq!(mesh.colors[b],rig.mesh.colors[a]);assert_eq!(joints[b],rig.ids[a]);assert_eq!(weights[b],rig.weights[a]);
                if a==b {assert_eq!(mesh.uvs[b],rig.mesh.uvs[a]);continue;}
                changed[usize::from(mesh.positions[b][0]>0.)]+=1;
                let [u,v]=mesh.uvs[b];let x=(u*painting.size.0 as f32) as u32;let y=(v*painting.size.1 as f32) as u32;
                let rgb=&painting.rgba[((y*painting.size.0+x)*4) as usize..][..3];
                assert!(rgb[0]>100&&rgb[0]>rgb[1]*2&&rgb[0]>rgb[2]*2,"samples skin/collar/motif: {rgb:?}");
            }
            assert!(changed.iter().all(|&n|n>6),"missing plain regions: {changed:?}");
            let n=rig.mesh.positions.len();assert_eq!(&mesh.positions[..n],&rig.mesh.positions);
            assert_eq!(&mesh.uvs[..n],&rig.mesh.uvs);assert_eq!(mesh.indices.len(),rig.mesh.indices.len());
        }
    }

    #[test]
    #[ignore = "requires supplied Kimono Girl painting and female rig"]
    fn supplied_kimono_underarms_sample_unbroken_cloth_on_both_sides() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig = PersonRig::parse(&std::fs::read(root.join("people/kenney/human-female.glb")).unwrap()).unwrap();
        let painting = crate::npc_textures::decode(&std::fs::read(
            root.join("people/npc-textures/kimono_girl.png")).unwrap()).unwrap();
        let look = crate::npc_appearance::look(CharacterKind::KimonoGirl);
        let crate::open_people::RefinedBody { mut mesh, mut ids, mut weights } = rig.refined_body();
        crate::kimono_sleeves::sculpt(&mut mesh, &mut ids, &mut weights, look);
        crate::npc_appearance::correct_body_uvs(&mut mesh, look, &mut ids, &mut weights);
        map_back(&mut mesh, look, &mut ids, &mut weights);
        lab_coat_collar_edges(&mut mesh, look, &mut ids, &mut weights);
        let before = mesh.clone();
        let before_ids = ids.clone();
        let before_weights = weights.clone();
        kimono_shoulders(&mut mesh, look, &mut ids, &mut weights);
        let mut covered = [[0; 2]; 2];
        let mut old_motif = 0;
        let red_cloth = |uv: [f32; 2]| {
            let x = (uv[0] * painting.size.0 as f32) as u32;
            let y = (uv[1] * painting.size.1 as f32) as u32;
            let rgb = &painting.rgba[((y * painting.size.0 + x) * 4) as usize..][..3];
            rgb[0] > 100 && u16::from(rgb[0]) > 2 * u16::from(rgb[1])
                && u16::from(rgb[0]) > 2 * u16::from(rgb[2])
        };
        for (old, new) in before.indices.chunks_exact(3).zip(mesh.indices.chunks_exact(3)) {
            for (&a, &b) in old.iter().zip(new) {
                let (a, b) = (a as usize, b as usize);
                assert_eq!(before.positions[a], mesh.positions[b]);
                assert_eq!(before.normals[a], mesh.normals[b]);
                assert_eq!(before.colors[a], mesh.colors[b]);
                assert_eq!(before_ids[a], ids[b]);
                assert_eq!(before_weights[a], weights[b]);
            }
            // Sample the actual prepared recess below the shoulder cap,
            // independently of the mapper's whole-face selection thresholds.
            let underarm = old.iter().all(|&i| {
                let p = before.positions[i as usize];
                (0.36..0.49).contains(&p[0].abs()) && (2.17..2.28).contains(&p[1])
            });
            if underarm {
                let p = before.positions[old[0] as usize];
                covered[usize::from(p[0] > 0.)][usize::from(p[2] > 0.)] += 1;
                for a in 0..=8 { for b in 0..=8-a {
                    let bary = [a as f32/8., b as f32/8., (8-a-b) as f32/8.];
                    let sample = |surface: &SurfaceMeshData, face: &[u32]| {
                        std::array::from_fn(|axis| (0..3).map(|corner|
                            surface.uvs[face[corner] as usize][axis] * bary[corner]).sum())
                    };
                    old_motif += usize::from(!red_cloth(sample(&before, old)));
                    assert!(red_cloth(sample(&mesh, new)), "underarm crosses motif/separator at {p:?}");
                }}
            }
            // Keep the front portrait/collar, exposed skin and lower body.
            if old.iter().all(|&i| {
                let p = before.positions[i as usize];
                p[1] < 2.30 && p[0].abs() < 0.28
            }) || old.iter().all(|&i| before.uvs[i as usize][0] >= 0.625) {
                for (&a, &b) in old.iter().zip(new) {
                    assert_eq!(before.uvs[a as usize], mesh.uvs[b as usize]);
                }
            }
        }
        assert!(covered.iter().flatten().all(|&n| n > 4), "missing underarm quadrants: {covered:?}");
        assert!(old_motif > 20, "fixture must expose the old stray motif");
        assert_eq!(before.indices.len(), mesh.indices.len());
    }

    #[test]
    #[ignore = "requires supplied lab-coat paintings and rig in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_lab_coat_sleeves_use_plain_fabric_and_preserve_skinning() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig = PersonRig::parse(&std::fs::read(root.join("people/kenney/human-male.glb")).unwrap()).unwrap();
        for kind in [CharacterKind::Elm, CharacterKind::Oak, CharacterKind::Scientist, CharacterKind::Blaine, CharacterKind::Pharmacist, CharacterKind::Elder, CharacterKind::Sage, CharacterKind::Pryce, CharacterKind::Kurt, CharacterKind::KurtOutside, CharacterKind::Blue] {
            let painting_key = if kind == CharacterKind::Blue { "elm" } else { crate::npc_appearance::look(kind).texture_key };
            let painting = crate::npc_textures::decode(&std::fs::read(root.join(format!("people/npc-textures/{painting_key}.png"))).unwrap()).unwrap();
            let mut mesh = rig.mesh.clone();
            let mut ids = rig.ids.clone();
            let mut weights = rig.weights.clone();
            long_sleeves(&mut mesh, &mut ids, &mut weights, crate::npc_appearance::look(kind));
            let mut changed = [0;2];
            for (&a,&b) in rig.mesh.indices.iter().zip(&mesh.indices) {
                let (a,b) = (a as usize,b as usize);
                assert_eq!(mesh.positions[b],rig.mesh.positions[a]);
                assert_eq!(mesh.normals[b],rig.mesh.normals[a]);
                assert_eq!(mesh.colors[b],rig.mesh.colors[a]);
                assert_eq!(ids[b],rig.ids[a]); assert_eq!(weights[b],rig.weights[a]);
                if a==b {assert_eq!(mesh.uvs[b],rig.mesh.uvs[a]);continue;}
                changed[usize::from(mesh.positions[b][0]>0.)]+=1;
                let [u,v]=mesh.uvs[b];
                let x=(u*painting.size.0 as f32) as u32;
                let y=(v*painting.size.1 as f32) as u32;
                let rgb=&painting.rgba[((y*painting.size.0+x)*4) as usize..][..3];
                if matches!(kind, CharacterKind::Kurt | CharacterKind::KurtOutside) { assert!(rgb[2]>rgb[1] && rgb[1]>rgb[0] && rgb[2]<160,"indigo robe samples collar detail: {rgb:?}"); }
                else if kind == CharacterKind::Pryce { assert!(rgb[2]>160 && rgb[1]>120 && rgb[2]>rgb[0]+50,"cardigan samples collar detail: {rgb:?}"); }
                else if kind == CharacterKind::Sage { assert!(rgb[0]>160 && rgb[1]>140 && rgb[0]>rgb[2]+20,"robe samples collar: {rgb:?}"); }
                else if kind == CharacterKind::Elder { assert!(rgb[0]>rgb[1] && rgb[1]>rgb[2]+30,"robe sleeve samples detail: {rgb:?}"); }
                else { assert!(rgb.iter().all(|&v|v>190),"sleeve samples nonwhite artwork: {rgb:?}"); }
            }
            assert_eq!(mesh.indices.len(),rig.mesh.indices.len());
            if matches!(kind, CharacterKind::Elm | CharacterKind::Oak | CharacterKind::Scientist | CharacterKind::Blaine | CharacterKind::Pharmacist | CharacterKind::Elder | CharacterKind::Sage | CharacterKind::Pryce | CharacterKind::Kurt | CharacterKind::KurtOutside) {assert!(changed.iter().all(|&n|n>100));}
            else {assert_eq!(changed,[0;2]);assert_eq!(mesh.positions,rig.mesh.positions);}
        }
    }

    #[test]
    #[ignore = "requires supplied lab-coat paintings and rig in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_lab_coat_collar_edges_sample_plain_fabric_without_moving_skin() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig = PersonRig::parse(&std::fs::read(root.join("people/kenney/human-male.glb")).unwrap()).unwrap();
        for kind in [CharacterKind::Elm, CharacterKind::Oak, CharacterKind::Scientist, CharacterKind::Blaine, CharacterKind::Pharmacist, CharacterKind::Elder, CharacterKind::Sage, CharacterKind::Pryce, CharacterKind::Kurt, CharacterKind::KurtOutside, CharacterKind::PokefanM, CharacterKind::Blue] {
            let look = crate::npc_appearance::look(kind);
            let painting_key = if kind == CharacterKind::Blue { "elm" } else { look.texture_key };
            let painting = crate::npc_textures::decode(&std::fs::read(root.join(format!("people/npc-textures/{painting_key}.png"))).unwrap()).unwrap();
            let mut mesh = rig.mesh.clone(); let mut ids = rig.ids.clone(); let mut weights = rig.weights.clone();
            lab_coat_collar_edges(&mut mesh,look,&mut ids,&mut weights);
            let mut changed = [0;2];
            for (&a,&b) in rig.mesh.indices.iter().zip(&mesh.indices) {
                let (a,b) = (a as usize,b as usize);
                assert_eq!(mesh.positions[b],rig.mesh.positions[a]);
                assert_eq!(mesh.normals[b],rig.mesh.normals[a]);
                assert_eq!(mesh.colors[b],rig.mesh.colors[a]);
                assert_eq!(ids[b],rig.ids[a]); assert_eq!(weights[b],rig.weights[a]);
                if a==b {assert_eq!(mesh.uvs[b],rig.mesh.uvs[a]); continue;}
                assert!(rig.mesh.uvs[a][1]>0.60,"changed neck skin");
                changed[usize::from(mesh.positions[b][0]>0.)]+=1;
                let [u,v]=mesh.uvs[b];
                let x=(u*painting.size.0 as f32) as u32; let y=(v*painting.size.1 as f32) as u32;
                let rgb=&painting.rgba[((y*painting.size.0+x)*4) as usize..][..3];
                if kind == CharacterKind::PokefanM { assert!(rgb[0]>rgb[1] && rgb[1]>rgb[2] && rgb[0]<140,"polo sides sample collar: {rgb:?}"); }
                else if matches!(kind, CharacterKind::Kurt | CharacterKind::KurtOutside) { assert!(rgb[2]>rgb[1] && rgb[1]>rgb[0] && rgb[2]<160,"indigo robe samples collar detail: {rgb:?}"); }
                else if kind == CharacterKind::Pryce { assert!(rgb[2]>160 && rgb[1]>120 && rgb[2]>rgb[0]+50,"cardigan samples collar detail: {rgb:?}"); }
                else if kind == CharacterKind::Sage { assert!(rgb[0]>160 && rgb[1]>140 && rgb[0]>rgb[2]+20,"robe samples collar: {rgb:?}"); }
                else if kind == CharacterKind::Elder { assert!(rgb[0]>rgb[1] && rgb[1]>rgb[2]+30,"robe collar samples detail: {rgb:?}"); }
                else { assert!(rgb.iter().all(|&c|c>190),"{} collar samples front detail: {rgb:?}",look.texture_key); }
            }
            assert_eq!(mesh.indices.len(),rig.mesh.indices.len());
            if kind==CharacterKind::Blue {assert_eq!(changed,[0;2]);assert_eq!(mesh.positions,rig.mesh.positions);}
            else {assert!(changed.iter().all(|&n|n>6),"{} missing collar sides: {changed:?}",look.texture_key);}
        }
    }

    #[test]
    #[ignore = "requires supplied Clair and Sabrina paintings and rig in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_high_collar_shoulders_sample_fabric_instead_of_neck_skin() {
        use CharacterKind::{Clair, Sabrina};
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for kind in [Clair, Sabrina] {
            let look = crate::npc_appearance::look(kind);
            let rig = PersonRig::parse(&std::fs::read(root.join(format!("people/kenney/{}.glb", look.base))).unwrap()).unwrap();
            let painting = crate::npc_textures::decode(&std::fs::read(root.join(format!("people/npc-textures/{}.png", look.texture_key))).unwrap()).unwrap();
            let mut mesh = rig.mesh.clone();
            let mut joints = rig.ids.clone();
            let mut weights = rig.weights.clone();
            map_front(&mut mesh, look, &mut joints, &mut weights);
            let mut shoulders = [0; 2];
            for &index in &mesh.indices {
                let i = index as usize;
                let p = mesh.positions[i];
                if i < rig.mesh.positions.len() || p[2] <= 0.1
                    || p[1] < 2.3 || p[0].abs() < 0.16 { continue; }
                let [u,v] = mesh.uvs[i];
                let x = (u * painting.size.0 as f32) as u32;
                let y = (v * painting.size.1 as f32) as u32;
                let rgb = &painting.rgba[((y*painting.size.0+x)*4) as usize..][..3];
                let fabric = if kind == Clair { f32::from(rgb[2]) > f32::from(rgb[0])*1.3 }
                    else { rgb[0] < 160 && f32::from(rgb[0]) > f32::from(rgb[1])*1.8 };
                assert!(fabric, "{} shoulder samples skin: {rgb:?} at {u},{v}", look.texture_key);
                shoulders[usize::from(p[0] > 0.0)] += 1;
            }
            assert!(shoulders.iter().all(|&n| n > 6), "{} missing shoulder coverage: {shoulders:?}", look.texture_key);
        }
    }

    #[test]
    #[ignore = "requires supplied Red/Misty paintings and rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_short_sleeves_sample_shirts_and_skin_without_moving_the_rig() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for kind in [CharacterKind::Red, CharacterKind::Misty, CharacterKind::Brock, CharacterKind::PokefanM, CharacterKind::Nurse] {
        let look = crate::npc_appearance::look(kind);
        let rig = PersonRig::parse(&std::fs::read(root.join(format!("people/kenney/{}.glb",look.base))).unwrap()).unwrap();
        let painting = crate::npc_textures::decode(&std::fs::read(root.join(format!("people/npc-textures/{}.png",look.texture_key))).unwrap()).unwrap();
        let mut mesh = rig.mesh.clone();
        let mut ids = rig.ids.clone();
        let mut weights = rig.weights.clone();
        short_sleeves(&mut mesh, &mut ids, &mut weights, look);
        assert_eq!(mesh.indices.len(), rig.mesh.indices.len());
        let mut coverage = [[0; 2]; 2];
        for (&a, &b) in rig.mesh.indices.iter().zip(&mesh.indices) {
            let (a,b) = (a as usize,b as usize);
            assert_eq!(mesh.positions[b], rig.mesh.positions[a]);
            assert_eq!(mesh.normals[b], rig.mesh.normals[a]);
            assert_eq!(mesh.colors[b], rig.mesh.colors[a]);
            assert_eq!(ids[b], rig.ids[a]);
            assert_eq!(weights[b], rig.weights[a]);
            if a == b { assert_eq!(mesh.uvs[b], rig.mesh.uvs[a]); continue; }
            let [u,v] = mesh.uvs[b];
            let x = (u * painting.size.0 as f32) as u32;
            let y = (v * painting.size.1 as f32) as u32;
            let pixel = &painting.rgba[((y*painting.size.0+x)*4) as usize..][..3];
            let sleeve = u < 0.5;
            coverage[usize::from(mesh.positions[b][0]>0.0)][usize::from(sleeve)] += 1;
            if sleeve {
                let fabric = if matches!(kind, CharacterKind::Red | CharacterKind::Nurse) {pixel.iter().all(|&c| c>170)}
                    else if kind == CharacterKind::Misty {pixel[0]>170 && pixel[1]>130 && pixel[2]<140}
                    else if kind == CharacterKind::Brock {pixel[0]>130 && pixel[1]>50 && pixel[1]<180 && pixel[2]<130}
                    else {pixel[0]>pixel[1] && pixel[1]>pixel[2] && pixel[0]<140};
                assert!(fabric, "{} sleeve samples wrong fabric: {pixel:?}",look.texture_key);
            } else {
                if matches!(kind, CharacterKind::Brock | CharacterKind::PokefanM) {
                    assert!((0.67..=0.733).contains(&v), "Brock forearm leaves warm skin patch");
                }
                assert!(f32::from(pixel[0]) > f32::from(pixel[2])*1.2, "forearm missing warm skin: {pixel:?}");
            }
        }
        assert!(coverage.iter().flatten().all(|&n| n>50), "missing arm section: {coverage:?}");
        }
    }

    #[test]
    #[ignore = "requires supplied rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_front_clothing_mapping_preserves_geometry_skin_and_animation_weights() {
        use CharacterKind::*;
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for kind in [Trainer, TrainerFemale, Red, KimonoGirl, Mom, Jasmine, Karen, Beauty, Daisy, Clair, Sabrina, Erika, Bugsy, Receptionist, Whitney, Misty, Janine, Koga, Morty, Falkner, Lance, Rival, Blue, Brock, Elm, Oak, Scientist, Blaine, Pharmacist, Elder, Sage, Pryce, Kurt, KurtOutside, PokefanM, Lass, Nurse, CooltrainerF, SuperNerd, Gramps, Granny, Youngster, Officer] {
            let look = crate::npc_appearance::look(kind);
            let rig = PersonRig::parse(&std::fs::read(
                root.join(format!("people/kenney/{}.glb", look.base))).unwrap()).unwrap();
            let mut mesh = rig.mesh.clone();
            let mut ids = rig.ids.clone();
            let mut weights = rig.weights.clone();
            crate::npc_appearance::correct_body_uvs(&mut mesh, look, &mut ids, &mut weights);
            let old = mesh.clone();
            let old_ids = ids.clone();
            let old_weights = weights.clone();
            map_front(&mut mesh, look, &mut ids, &mut weights);
            let n = old.positions.len();
            assert_eq!(&mesh.positions[..n], &old.positions);
            assert_eq!(&mesh.normals[..n], &old.normals);
            assert_eq!(&mesh.colors[..n], &old.colors);
            assert_eq!(&mesh.uvs[..n], &old.uvs);
            assert_eq!(&ids[..n], &old_ids);
            assert_eq!(&weights[..n], &old_weights);
            assert_eq!(mesh.indices.len(), old.indices.len());
            let mut changed = 0;
            let mut collar = 0;
            for (&a, &b) in old.indices.iter().zip(&mesh.indices) {
                let a = a as usize;
                let b = b as usize;
                // Every triangle retains its exact positions, winding and
                // skin weights, so every animation deforms identically.
                assert_eq!(mesh.positions[b], old.positions[a]);
                assert_eq!(mesh.normals[b], old.normals[a]);
                assert_eq!(mesh.colors[b], old.colors[a]);
                assert_eq!(ids[b], old_ids[a]);
                assert_eq!(weights[b], old_weights[a]);
                if b != a {
                    changed += 1;
                    assert!(old.uvs[a][1] > 0.47 && old.uvs[a][0] < if kind == KimonoGirl {0.625} else {0.5});
                    if kind == KimonoGirl && old.positions[a][0].abs() >= 0.4 {
                        assert!(old.positions[a][1]>1.90 && old.uvs[a][1]>=0.475
                            && old.uvs[a][0]<0.625,"changed outer face outside shoulder cloth");
                    } else { assert!(old.positions[a][0].abs() < 0.4); }
                    assert!((0.0..1.0).contains(&mesh.uvs[b][0]));
                    assert!((0.47..1.0).contains(&mesh.uvs[b][1]));
                    collar += usize::from(mesh.positions[b][2] > 0.1 && mesh.uvs[b][1] < 0.55);
                }
            }
            if look.texture_key == "officer" {
                assert_eq!(changed, 0);
            } else {
                assert!(changed > 60, "{} missing front shirt", look.texture_key);
                assert!(collar > 6, "{} missing collar", look.texture_key);
            }
        }
    }
}
