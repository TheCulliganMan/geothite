//! Service colors are derived from the external pack, never a second palette catalog.
use super::*;
use bevy::{asset::RenderAssetUsages, render::render_resource::{Extent3d,TextureDimension}};

#[test]
fn service_uniforms_follow_all_actual_object_palettes_and_sprite_frames() {
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external verified pack")).unwrap();
    let pack=loaded.pack();
    let text=std::str::from_utf8(&pack.runtime_files()["gfx/overworld/npc_sprites.pal"]).unwrap();
    let banks=text.lines().filter_map(|l|l.trim().strip_prefix("RGB "))
        .map(|l|l.split(';').next().unwrap().split(',').map(|s|s.trim().parse::<u8>().unwrap())
            .map(|v|(v<<3)|(v>>2)).collect::<Vec<_>>()).collect::<Vec<_>>();
    assert_eq!(banks.len(),32);
    for (kind,name,key) in [(CharacterKind::Clerk,"clerk","SPRITE_CLERK"),
        (CharacterKind::Receptionist,"receptionist","SPRITE_RECEPTIONIST"),
        (CharacterKind::Lass,"lass","SPRITE_LASS"),
        (CharacterKind::Youngster,"youngster","SPRITE_YOUNGSTER"),
        (CharacterKind::CooltrainerM,"cooltrainer_m","SPRITE_COOLTRAINER_M"),
        (CharacterKind::SuperNerd,"super_nerd","SPRITE_SUPER_NERD"),
        (CharacterKind::Officer,"officer","SPRITE_OFFICER"),
        (CharacterKind::KimonoGirl,"kimono_girl","SPRITE_KIMONO_GIRL")] {
        let mut palettes=std::collections::BTreeSet::new();let mut objects=0;
        for map in pack.data().maps.values() {for object in &map.objects {
            if object.sprite!=key {continue;}
            let p=if object.pal!=0 {object.pal as i64} else {*pack.data().sprite_palette_defaults.get(key).unwrap()};
            palettes.insert((p&7)as usize);objects+=1;
        }}
        assert!(objects>0);println!("{name}: {objects} objects, palettes {palettes:?}");
        let source=image::load_from_memory(&pack.runtime_files()[&format!("gfx/sprites/{name}.png")]).unwrap().to_rgba8();
        assert_eq!(source.dimensions(),(16,96));
        for palette in palettes {for bank in 0..4 {for frame in 0..6 {
            let colors=&banks[bank*8+palette];
            // Same border-connected white transparency as source sprite publication.
            let mut transparent=[false;256];let mut queue=std::collections::VecDeque::new();
            for i in 0..256 {if (i%16==0||i%16==15||i<16||i>=240)&&source.get_pixel(i as u32%16,frame*16+i as u32/16)[0]==255 {
                transparent[i]=true;queue.push_back(i);
            }}
            while let Some(i)=queue.pop_front() {for j in [i.checked_sub(16), (i+16<256).then_some(i+16),
                (i%16>0).then(||i-1),(i%16<15).then_some(i+1)].into_iter().flatten() {
                if !transparent[j]&&source.get_pixel(j as u32%16,frame*16+j as u32/16)[0]==255 {transparent[j]=true;queue.push_back(j);}
            }}
            let mut pixels=Vec::new();
            for i in 0..256 {let shade=source.get_pixel(i as u32%16,frame*16+i as u32/16)[0];
                let at=match shade {255=>0,170=>1,85=>2,0=>3,_=>panic!("source shade")};
                pixels.extend_from_slice(&colors[at*3..at*3+3]);pixels.push(if transparent[i] {0} else {255});
            }
            let image=Image::new(Extent3d{width:16,height:16,depth_or_array_layers:1},TextureDimension::D2,pixels,
                TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::all());
            assert_eq!(source_color(kind,Some(&image)),Some(colors[6..9].try_into().unwrap()),"{name} palette {palette} bank {bank} frame {frame}");
        }}}
    }
}

#[test]
fn community_clothing_keeps_hair_faces_ink_and_dark_trousers() {
    for kind in [CharacterKind::Lass,CharacterKind::Youngster,CharacterKind::CooltrainerM,CharacterKind::SuperNerd,CharacterKind::Officer] {
        let look=crate::npc_appearance::look(kind);let mut pixels=vec![0;256*256*4];
        let set=|p:&mut Vec<u8>,x:usize,y:usize,c:[u8;4]|p[(y*256+x)*4..(y*256+x)*4+4].copy_from_slice(&c);
        let rgba=|c:[u8;3]|[c[0],c[1],c[2],255];
        set(&mut pixels,20,160,rgba(look.shirt));
        set(&mut pixels,200,220,rgba(look.lower));
        set(&mut pixels,20,20,rgba(look.hair));
        set(&mut pixels,80,60,[255,156,82,255]);
        set(&mut pixels,40,160,[247,247,227,255]);
        set(&mut pixels,50,160,[16,16,16,255]);
        set(&mut pixels,60,160,[look.shirt[0],look.shirt[1],look.shirt[2],0]);
        let before=pixels.clone();let color=[57,189,24];
        let result=painting_for(kind,&pixels,(256,256),color);
        assert_eq!(pixels,before);
        for i in 0..256*256 {let at=i*4;let xy=(i%256,i/256);
            if xy==(20,160)||(matches!(kind,CharacterKind::Lass|CharacterKind::CooltrainerM|CharacterKind::Officer)&&xy==(200,220)) {
                assert_eq!(&result[at..at+4],&rgba(color),"{kind:?}: clothing follows its actual source accent");
            }else {assert_eq!(&result[at..at+4],&before[at..at+4],"{kind:?}: protected role at {xy:?}");}
        }
    }
}

#[test]
fn kimono_palette_preserves_painted_flowers_collar_skin_ink_and_fold_shading() {
    let kind = CharacterKind::KimonoGirl;
    let mut pixels = vec![0;256*256*4];
    let samples = [
        (20,160,[209,56,59,255],Some([57,189,24,255])),
        (30,160,[104,20,24,255],Some([28,94,12,255])),
        (20,121,[209,56,59,255],Some([57,189,24,255])), // collar-side robe
        (20,119,[209,56,59,255],None), // protected upper island
        (200,220,[145,31,46,255],Some([57,189,24,255])),
        (40,160,[230,150,165,255],None), // pink blossom
        (50,160,[70,95,50,255],None), // stem
        (60,160,[245,245,230,255],None), // pale collar
        (70,160,[252,200,165,255],None), // skin
        (80,160,[32,24,24,255],None), // ink
        (90,160,[209,56,59,0],None), // transparent cloth
        (20,20,[209,56,59,255],None), // protected head island
        (200,160,[209,56,59,255],None), // protected hand/shoe island
    ];
    for &(x,y,rgba,_) in &samples {
        pixels[(y*256+x)*4..(y*256+x)*4+4].copy_from_slice(&rgba);
    }
    let original = pixels.clone();
    let painted = painting_for(kind,&pixels,(256,256),[57,189,24]);
    assert_eq!(pixels,original);
    for &(x,y,rgba,expected) in &samples {
        assert_eq!(&painted[(y*256+x)*4..(y*256+x)*4+4],&expected.unwrap_or(rgba),"{x},{y}");
    }
}

#[test]
fn service_uniform_painting_changes_only_clothing_and_keeps_all_other_roles() {
    for kind in [CharacterKind::Clerk,CharacterKind::Receptionist] {
        let look=crate::npc_appearance::look(kind);let mut pixels=vec![0;256*256*4];
        let set=|p:&mut Vec<u8>,x:usize,y:usize,c:[u8;4]|p[(y*256+x)*4..(y*256+x)*4+4].copy_from_slice(&c);
        let c=[look.shirt[0],look.shirt[1],look.shirt[2],255];
        for (x,y) in [(20,160),(200,220),(80,60)] {set(&mut pixels,x,y,c);}
        for (x,y,c) in [(30,160,[255,156,82,255]),(40,160,[245,245,230,255]),
            (50,160,[32,24,24,255]),(60,160,[94,156,74,0])] {set(&mut pixels,x,y,c);}
        let before=pixels.clone();let color=[82,74,255];let painted=painting_for(kind,&pixels,(256,256),color);
        assert_eq!(pixels,before);
        for i in 0..256*256 {let (x,y)=(i%256,i/256);let at=i*4;
            if (x,y)==(20,160)||(kind==CharacterKind::Receptionist&&(x,y)==(200,220)) {
                assert_eq!(&painted[at..at+4],&[82,74,255,255]);
            }else {assert_eq!(&painted[at..at+4],&before[at..at+4]);}
        }
    }
}
