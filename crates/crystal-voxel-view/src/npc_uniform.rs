//! Reviewed per-instance clothing roles follow the already paletted source sprite.
//! Keep the immutable external painting and all non-clothing UV islands intact.
use bevy::prelude::Image;
use bevy::render::render_resource::TextureFormat;
use crate::new_bark_actors::character_meshes::CharacterKind;

pub(crate) fn supported(kind: CharacterKind) -> bool {
    matches!(kind, CharacterKind::LinkReceptionist | CharacterKind::Fisher | CharacterKind::CooltrainerF | CharacterKind::Clerk | CharacterKind::Receptionist
        | CharacterKind::Lass | CharacterKind::Youngster | CharacterKind::CooltrainerM | CharacterKind::SuperNerd | CharacterKind::Officer)
}
pub(crate) fn lower_follows_source(kind: CharacterKind) -> bool {
    // Lass skirts and Cooltrainer/Officer trousers use source shade two;
    // Youngster/Super Nerd trousers use the shared dark ink instead.
    matches!(kind,CharacterKind::LinkReceptionist|CharacterKind::Receptionist|CharacterKind::Lass|CharacterKind::CooltrainerM|CharacterKind::Officer)
}
pub(crate) fn source_color(kind: CharacterKind, image: Option<&Image>) -> Option<[u8; 3]> {
    if !supported(kind) { return None; }
    let image = image?;
    if !matches!(image.texture_descriptor.format, TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb) {
        return None;
    }
    let mut colors = std::collections::BTreeMap::<[u8; 3], usize>::new();
    for p in image.data.as_deref()?.chunks_exact(4) {
        // Reviewed green/blue clothing colors are distinct from warm skin,
        // paper, transparent pixels and the shared dark outline.
        let cool = u16::from(p[1].max(p[2])) > u16::from(p[0]) + 24 && p[1].abs_diff(p[2]) >= 16;
        // The reviewed red accent is separate from the warm orange skin
        // color. Pink link paintings retain their existing fallback.
        let red = kind != CharacterKind::LinkReceptionist
            && u16::from(p[0]) > u16::from(p[1].max(p[2])) + 24
            && u16::from(p[1]) * 5 <= u16::from(p[0]) * 2;
        let single_pigment = matches!(kind, CharacterKind::Clerk | CharacterKind::Receptionist
            | CharacterKind::Lass | CharacterKind::Youngster | CharacterKind::CooltrainerM | CharacterKind::SuperNerd | CharacterKind::Officer);
        // These sprites use one clothing pigment, warm skin and dark ink.
        // Include their brown/silver/pink uniforms as well as red/blue/green.
        let pigment = p[..3].iter().copied().max().unwrap() > 64
            && p[..3].iter().copied().min().unwrap() < 180
            && !(p[0] > 180 && p[1] > 110 && p[2] > 40 && p[0] > p[1] && p[1] > p[2]);
        if p[3] < 128 || !(if single_pigment { pigment } else { cool || red }) { continue; }
        *colors.entry([p[0], p[1], p[2]]).or_default() += 1;
    }
    colors.into_iter().max_by_key(|(rgb, count)| (*count, *rgb)).map(|(rgb, _)| rgb)
}

pub(crate) fn painting(rgba: &[u8], size: (u32, u32), color: [u8; 3]) -> Vec<u8> {
    let mut out = rgba.to_vec();
    if size.0 == 0 || size.1 == 0 { return out; }
    for (i, p) in out.chunks_exact_mut(4).enumerate() {
        let x = (i as u32 % size.0) * 256 / size.0;
        let y = (i as u32 / size.0) * 256 / size.1;
        let clothing = (x < 155 && y >= 128) || (x >= 156 && y >= 192);
        // Preserve the collar, badge, seams and ink. Only the pink cloth in
        // the documented shirt/trouser islands changes hue; never tint a face.
        if !clothing || p[3] == 0 || p[0].saturating_sub(p[2]) < 10
            || p[2].saturating_sub(p[1]) < 10 { continue; }
        let shade = f32::from(p[0]) / 230.;
        for channel in 0..3 { p[channel] = (f32::from(color[channel]) * shade).round().clamp(0., 255.) as u8; }
    }
    out
}

/// Repaint only each reviewed sprite's clothing roles. Dark trousers remain
/// intact unless their source uses the palette accent; hair changes only for
/// the female Cooltrainer, whose source explicitly shares that accent.
pub(crate) fn painting_for(kind: CharacterKind, rgba: &[u8], size: (u32,u32), color: [u8;3]) -> Vec<u8> {
    if kind == CharacterKind::LinkReceptionist { return painting(rgba,size,color); }
    let mut out=rgba.to_vec();
    if !supported(kind) || size.0==0 || size.1==0 { return out; }
    let look=crate::npc_appearance::look(kind);
    for (i,p) in out.chunks_exact_mut(4).enumerate() {
        if p[3]==0 { continue; }
        let x=(i as u32%size.0)*256/size.0;let y=(i as u32/size.0)*256/size.1;
        let lower = lower_follows_source(kind) && x>=156 && y>=192;
        let garment = (x<155 && y>=128) || lower;
        let green = kind==CharacterKind::Clerk;
        let red = kind==CharacterKind::Lass;
        let shirt=garment && if green { p[1].saturating_sub(p[0].max(p[2]))>=12 }
            else if red { p[0].saturating_sub(p[1].max(p[2]))>=12 }
            else { p[2].saturating_sub(p[0].max(p[1]))>=12 };
        let hair=kind==CharacterKind::CooltrainerF && x<160
            && ((y<125 && !(48..118).contains(&x)) || y<48 || (y<56 && (x<60||x>=100)))
            && p[0]<125 && p[1]<120 && p[2]<115 && p[0].saturating_sub(p[1])>5 && p[1]>p[2];
        let channel=if green {1} else if red {0} else {2};
        let shade=if shirt {f32::from(p[channel])/f32::from(if lower {look.lower[channel]} else {look.shirt[channel]})}
            else if hair {f32::from(p[0])/f32::from(look.hair[0])} else {continue;};
        for channel in 0..3 {p[channel]=(f32::from(color[channel])*shade).round().clamp(0.,255.)as u8;}
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{asset::RenderAssetUsages, render::render_resource::{Extent3d, TextureDimension}};
    pub(super) fn sprite(color: [u8; 3]) -> Image {
        Image::new_fill(Extent3d { width:16,height:16,depth_or_array_layers:1 },TextureDimension::D2,
            &[color[0],color[1],color[2],255],TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::all())
    }
    #[test]
    fn link_uniform_uses_each_actual_sprite_palette_and_ignores_skin_and_transparency() {
        for color in [[80,192,48],[48,104,224],[24,88,32]] {
            let mut image=sprite(color);
            let data=image.data.as_mut().unwrap();
            for p in data[..256].chunks_exact_mut(4) {p.copy_from_slice(&[248,168,80,255]);}
            for p in data[256..512].chunks_exact_mut(4) {p.copy_from_slice(&[0,255,255,0]);}
            assert_eq!(source_color(CharacterKind::LinkReceptionist,Some(&image)),Some(color));
            assert_eq!(source_color(CharacterKind::Nurse,Some(&image)),None);
        }
        assert_eq!(source_color(CharacterKind::LinkReceptionist,None),None);
        assert_eq!(source_color(CharacterKind::LinkReceptionist,Some(&sprite([230,107,145]))),None);
    }
    #[test]
    fn link_uniform_painting_preserves_face_hair_cap_badge_alpha_and_immutable_pixels() {
        let mut rgba=vec![0;256*256*4];
        for p in rgba.chunks_exact_mut(4) {p.copy_from_slice(&[230,107,145,255]);}
        let set=|pixels:&mut Vec<u8>,x:usize,y:usize,color:[u8;4]|pixels[(y*256+x)*4..(y*256+x)*4+4].copy_from_slice(&color);
        set(&mut rgba,20,150,[255,255,240,255]); // white collar/badge
        set(&mut rgba,30,150,[32,24,24,255]); // seam
        set(&mut rgba,40,150,[230,107,145,0]); // transparency
        let original=rgba.clone();let green=painting(&rgba,(256,256),[80,192,48]);
        let blue=painting(&rgba,(256,256),[48,104,224]);
        assert_eq!(rgba,original);
        for y in 0..256 {for x in 0..256 {
            let i=(y*256+x)*4;let before=&rgba[i..i+4];let after=&green[i..i+4];
            if (x<155&&y>=128)||(x>=156&&y>=192) {
                if before==[230,107,145,255] {assert_eq!(after,[80,192,48,255]);}
                else {assert_eq!(after,before);}
            } else {assert_eq!(after,before);}
            assert_eq!(after[3],before[3]);
        }}
        assert_ne!(green,blue);
    }
    #[test]
    fn reviewed_uniform_source_colors_separate_red_accents_and_green_cloth_from_skin() {
        for kind in [CharacterKind::Fisher,CharacterKind::CooltrainerF] {
            for color in [[57,189,24],[255,57,8],[48,104,224]] {
                let mut image=sprite(color);
                for p in image.data.as_mut().unwrap()[..512].chunks_exact_mut(4) {p.copy_from_slice(&[255,156,82,255]);}
                assert_eq!(source_color(kind,Some(&image)),Some(color));
            }
            assert_eq!(source_color(kind,Some(&sprite([255,156,82]))),None);
        }
    }
    #[test]
    fn reviewed_uniform_roles_preserve_lower_garments_faces_badges_and_alpha() {
        let mut rgba=vec![0;256*256*4];
        let set=|pixels:&mut Vec<u8>,x:usize,y:usize,c:[u8;4]|pixels[(y*256+x)*4..(y*256+x)*4+4].copy_from_slice(&c);
        set(&mut rgba,20,160,[71,107,186,255]); // shirt
        set(&mut rgba,200,220,[43,71,107,255]); // dark lower garment
        set(&mut rgba,80,60,[76,43,31,255]); // face detail inside protected island
        set(&mut rgba,20,20,[76,43,31,255]); // hair outside face
        set(&mut rgba,40,160,[245,245,230,255]); // badge
        set(&mut rgba,50,160,[71,107,186,0]); // transparency
        let original=rgba.clone();let color=[255,57,8];
        for kind in [CharacterKind::Fisher,CharacterKind::CooltrainerF] {
            let painted=painting_for(kind,&rgba,(256,256),color);
            for y in 0..256 {for x in 0..256 {
                let i=(y*256+x)*4;
                let changed=(x,y)==(20,160)||(kind==CharacterKind::CooltrainerF&&(x,y)==(20,20));
                if changed {assert_eq!(&painted[i..i+4],&[255,57,8,255]);}
                else {assert_eq!(&painted[i..i+4],&rgba[i..i+4]);}
            }}
        }
        assert_eq!(rgba,original);
        assert_eq!(painting_for(CharacterKind::Nurse,&rgba,(256,256),color),original);
    }

}

#[cfg(test)]
#[path = "npc_uniform_service_tests.rs"]
mod service_tests;
