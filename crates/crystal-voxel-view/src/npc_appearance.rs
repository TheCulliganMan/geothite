//! Appearance adaptations for the source NPC identities on the external rig.
//! Geometry comes from the weighted CC0 body; costume details remain original
//! renderer code. No sprite, texture or converted model is embedded here.
use crate::new_bark_actors::character_meshes::CharacterKind;
#[derive(Clone, Copy)]
pub(crate) struct Look {
    pub base: &'static str,
    pub texture_key: &'static str,
    pub shirt: [u8; 3],
    pub lower: [u8; 3],
    pub hair: [u8; 3],
    pub hairstyle: &'static str,
    pub hat: &'static str,
    pub costume: &'static str,
    pub accessories: &'static str,
    pub width: f32,
    pub height: f32,
}
pub(crate) fn look(kind: CharacterKind) -> Look {
    use CharacterKind::*;
    match kind {
        Beauty => Look { base: "human-female", texture_key: "beauty", shirt: [71, 107, 186], lower: [71, 107, 186], hair: [76, 43, 31], hairstyle: "long", hat: "none", costume: "dress", accessories: "necklace,earrings", width: 0.91, height: 1.38 },
        Biker => Look { base: "human-male", texture_key: "biker", shirt: [33, 46, 64], lower: [71, 110, 138], hair: [48, 33, 28], hairstyle: "sidecut", hat: "none", costume: "vest", accessories: "stubble,chain,gloves", width: 1.22, height: 1.38 },
        Bill => Look { base: "human-male", texture_key: "bill", shirt: [209, 56, 59], lower: [76, 43, 31], hair: [76, 43, 31], hairstyle: "curls", hat: "none", costume: "polo", accessories: "pen", width: 0.95, height: 1.38 },
        BlackBelt => Look { base: "human-male", texture_key: "black_belt", shirt: [33, 46, 64], lower: [247, 247, 227], hair: [48, 33, 28], hairstyle: "spikes", hat: "none", costume: "martial", accessories: "headband,blackbelt", width: 1.08, height: 1.38 },
        Blaine => Look { base: "human-male", texture_key: "blaine", shirt: [247, 247, 227], lower: [145, 31, 46], hair: [173, 184, 181], hairstyle: "bald", hat: "none", costume: "coat", accessories: "glasses,moustache,tie", width: 1.08, height: 1.38 },
        Blue => Look { base: "human-male", texture_key: "blue", shirt: [99, 64, 125], lower: [43, 71, 107], hair: [76, 43, 31], hairstyle: "spikes", hat: "none", costume: "tee", accessories: "pendant", width: 1.01, height: 1.38 },
        Brock => Look { base: "human-male", texture_key: "brock", shirt: [89, 122, 76], lower: [76, 43, 31], hair: [76, 43, 31], hairstyle: "spikes", hat: "none", costume: "vest", accessories: "hikingpack", width: 1.08, height: 1.38 },
        Bruno => Look { base: "human-male", texture_key: "bruno", shirt: [252, 196, 145], lower: [247, 247, 227], hair: [48, 33, 28], hairstyle: "topknot", hat: "none", costume: "martial", accessories: "blackbelt,wristwraps", width: 1.25, height: 1.38 },
        BugCatcher => Look { base: "human-male", texture_key: "bug_catcher", shirt: [71, 107, 186], lower: [166, 166, 102], hair: [76, 43, 31], hairstyle: "short", hat: "straw", costume: "tee", accessories: "shorts,net", width: 0.88, height: 1.05 },
        Bugsy => Look { base: "human-male", texture_key: "bugsy", shirt: [94, 156, 74], lower: [166, 166, 102], hair: [99, 64, 125], hairstyle: "bob", hat: "none", costume: "polo", accessories: "shorts,netbelt", width: 0.88, height: 1.05 },
        Cal => Look { base: "human-male", texture_key: "cal", shirt: [209, 56, 59], lower: [33, 46, 64], hair: [76, 43, 31], hairstyle: "short", hat: "bluecap", costume: "jacket", accessories: "backpack", width: 1.00, height: 1.38 },
        Captain => Look { base: "human-male", texture_key: "captain", shirt: [43, 71, 107], lower: [43, 71, 107], hair: [173, 184, 181], hairstyle: "short", hat: "captain", costume: "suit", accessories: "moustache,goldbuttons", width: 1.18, height: 1.38 },
        Trainer => Look { base: "human-male", texture_key: "chris", shirt: [209, 56, 59], lower: [43, 71, 107], hair: [48, 33, 28], hairstyle: "short", hat: "trainer", costume: "jacket", accessories: "backpack", width: 1.00, height: 1.20 },
        Chuck => Look { base: "human-male", texture_key: "chuck", shirt: [252, 196, 145], lower: [247, 247, 227], hair: [76, 43, 31], hairstyle: "short", hat: "none", costume: "martial", accessories: "beard,blackbelt", width: 1.36, height: 1.38 },
        Clair => Look { base: "human-female", texture_key: "clair", shirt: [71, 107, 186], lower: [71, 107, 186], hair: [71, 107, 186], hairstyle: "long", hat: "none", costume: "bodysuit", accessories: "cape,gloves,boots", width: 0.95, height: 1.38 },
        Clerk => Look { base: "human-male", texture_key: "clerk", shirt: [94, 156, 74], lower: [33, 46, 64], hair: [76, 43, 31], hairstyle: "short", hat: "none", costume: "apron", accessories: "namebadge", width: 0.96, height: 1.38 },
        CooltrainerF => Look { base: "human-female", texture_key: "cooltrainer_f", shirt: [71, 107, 186], lower: [43, 71, 107], hair: [76, 43, 31], hairstyle: "ponytail", hat: "none", costume: "jacket", accessories: "skirt,satchel", width: 0.94, height: 1.38 },
        CooltrainerM => Look { base: "human-male", texture_key: "cooltrainer_m", shirt: [71, 107, 186], lower: [43, 71, 107], hair: [48, 33, 28], hairstyle: "spikes", hat: "none", costume: "jacket", accessories: "wristband", width: 1.04, height: 1.38 },
        Daisy => Look { base: "human-female", texture_key: "daisy", shirt: [71, 107, 186], lower: [71, 107, 186], hair: [201, 156, 89], hairstyle: "long", hat: "none", costume: "dress", accessories: "ribbon,necklace", width: 0.93, height: 1.38 },
        Elder => Look { base: "human-male", texture_key: "elder", shirt: [201, 156, 89], lower: [76, 43, 31], hair: [173, 184, 181], hairstyle: "bald", hat: "none", costume: "robe", accessories: "beard,cane", width: 0.96, height: 1.23 },
        Elm => Look { base: "human-male", texture_key: "elm", shirt: [247, 247, 227], lower: [76, 43, 31], hair: [76, 43, 31], hairstyle: "spikes", hat: "none", costume: "coat", accessories: "glasses,tealtie,pen", width: 1.04, height: 1.38 },
        Erika => Look { base: "human-female", texture_key: "erika", shirt: [161, 59, 97], lower: [161, 59, 97], hair: [48, 33, 28], hairstyle: "bob", hat: "none", costume: "kimono", accessories: "flower,obi", width: 0.92, height: 1.38 },
        Falkner => Look { base: "human-male", texture_key: "falkner", shirt: [71, 107, 186], lower: [71, 107, 186], hair: [43, 71, 107], hairstyle: "sidepart", hat: "none", costume: "kimono", accessories: "obi", width: 0.98, height: 1.38 },
        Fisher => Look { base: "human-male", texture_key: "fisher", shirt: [71, 107, 186], lower: [43, 71, 107], hair: [76, 43, 31], hairstyle: "short", hat: "redcap", costume: "vest", accessories: "rod", width: 1.17, height: 1.38 },
        FishingGuru => Look { base: "human-male", texture_key: "fishing_guru", shirt: [71, 107, 186], lower: [43, 71, 107], hair: [173, 184, 181], hairstyle: "short", hat: "straw", costume: "vest", accessories: "moustache,rod", width: 1.25, height: 1.38 },
        GameboyKid => Look { base: "human-male", texture_key: "gameboy_kid", shirt: [71, 107, 186], lower: [166, 166, 102], hair: [48, 33, 28], hairstyle: "short", hat: "none", costume: "tee", accessories: "shorts,handheld", width: 0.86, height: 1.05 },
        Gentleman => Look { base: "human-male", texture_key: "gentleman", shirt: [43, 71, 107], lower: [43, 71, 107], hair: [173, 184, 181], hairstyle: "short", hat: "top", costume: "suit", accessories: "moustache,cane,bowtie", width: 1.05, height: 1.38 },
        Gramps => Look { base: "human-male", texture_key: "gramps", shirt: [204, 186, 143], lower: [76, 43, 31], hair: [173, 184, 181], hairstyle: "bald", hat: "none", costume: "cardigan", accessories: "moustache,glasses", width: 1.13, height: 1.23 },
        Granny => Look { base: "human-female", texture_key: "granny", shirt: [130, 104, 80], lower: [130, 104, 80], hair: [173, 184, 181], hairstyle: "bun", hat: "none", costume: "dress", accessories: "shawl,glasses", width: 1.15, height: 1.23 },
        GymGuide => Look { base: "human-male", texture_key: "gym_guide", shirt: [71, 107, 186], lower: [43, 71, 107], hair: [48, 33, 28], hairstyle: "short", hat: "none", costume: "suit", accessories: "tie,clipboard", width: 1.08, height: 1.38 },
        Janine => Look { base: "human-female", texture_key: "janine", shirt: [99, 64, 125], lower: [99, 64, 125], hair: [140, 97, 178], hairstyle: "highpony", hat: "none", costume: "ninja", accessories: "scarf,gloves", width: 0.93, height: 1.38 },
        Jasmine => Look { base: "human-female", texture_key: "jasmine", shirt: [247, 247, 227], lower: [247, 247, 227], hair: [76, 43, 31], hairstyle: "long", hat: "none", costume: "dress", accessories: "ribbon", width: 0.90, height: 1.38 },
        Karen => Look { base: "human-female", texture_key: "karen", shirt: [33, 46, 64], lower: [33, 46, 64], hair: [173, 184, 181], hairstyle: "long", hat: "none", costume: "dress", accessories: "necklace,bracelet", width: 0.93, height: 1.38 },
        KimonoGirl => Look { base: "human-female", texture_key: "kimono_girl", shirt: [209, 56, 59], lower: [145, 31, 46], hair: [48, 33, 28], hairstyle: "bun", hat: "none", costume: "kimono", accessories: "obi,fan,hairpin", width: 0.98, height: 1.38 },
        Koga => Look { base: "human-male", texture_key: "koga", shirt: [43, 71, 107], lower: [43, 71, 107], hair: [48, 33, 28], hairstyle: "spikes", hat: "none", costume: "ninja", accessories: "scarf,wristwraps", width: 1.07, height: 1.38 },
        TrainerFemale => Look { base: "human-female", texture_key: "kris", shirt: [43, 71, 107], lower: [43, 71, 107], hair: [54, 156, 153], hairstyle: "pigtails", hat: "kris", costume: "jacket", accessories: "backpack", width: 0.94, height: 1.20 },
        Kurt => Look { base: "human-male", texture_key: "kurt", shirt: [43, 71, 107], lower: [43, 71, 107], hair: [173, 184, 181], hairstyle: "sparse", hat: "none", costume: "kimono", accessories: "apronfront", width: 1.13, height: 1.23 },
        KurtOutside => Look { base: "human-male", texture_key: "kurt_outside", shirt: [43, 71, 107], lower: [43, 71, 107], hair: [173, 184, 181], hairstyle: "sparse", hat: "none", costume: "kimono", accessories: "apronfront,apricornbasket", width: 1.13, height: 1.23 },
        Lance => Look { base: "human-male", texture_key: "lance", shirt: [145, 31, 46], lower: [33, 46, 64], hair: [145, 31, 46], hairstyle: "spikes", hat: "none", costume: "suit", accessories: "redcape,goldbuttons,boots", width: 1.10, height: 1.38 },
        Lass => Look { base: "human-female", texture_key: "lass", shirt: [209, 56, 59], lower: [145, 31, 46], hair: [76, 43, 31], hairstyle: "ponytail", hat: "none", costume: "polo", accessories: "skirt,schoolbag", width: 0.90, height: 1.05 },
        LinkReceptionist => Look { base: "human-female", texture_key: "link_receptionist", shirt: [230, 107, 145], lower: [230, 107, 145], hair: [76, 43, 31], hairstyle: "bob", hat: "reception", costume: "dress", accessories: "namebadge", width: 0.96, height: 1.38 },
        Misty => Look { base: "human-female", texture_key: "misty", shirt: [245, 176, 54], lower: [71, 110, 138], hair: [245, 125, 48], hairstyle: "sidepony", hat: "none", costume: "tee", accessories: "shorts,suspenders", width: 0.91, height: 1.38 },
        Mom => Look { base: "human-female", texture_key: "mom", shirt: [209, 56, 59], lower: [145, 31, 46], hair: [76, 43, 31], hairstyle: "bob", hat: "none", costume: "dress", accessories: "apronfront", width: 1.09, height: 1.38 },
        Morty => Look { base: "human-male", texture_key: "morty", shirt: [99, 64, 125], lower: [204, 186, 143], hair: [201, 156, 89], hairstyle: "sidepart", hat: "none", costume: "tee", accessories: "headband,scarf", width: 1.00, height: 1.38 },
        Nurse => Look { base: "human-female", texture_key: "nurse", shirt: [247, 247, 227], lower: [247, 247, 227], hair: [230, 107, 145], hairstyle: "rolls", hat: "nurse", costume: "dress", accessories: "apronfront,medallion", width: 1.00, height: 1.38 },
        Oak => Look { base: "human-male", texture_key: "oak", shirt: [245, 227, 168], lower: [201, 156, 89], hair: [173, 184, 181], hairstyle: "spikes", hat: "none", costume: "coat", accessories: "redshirt,pen", width: 1.10, height: 1.38 },
        Officer => Look { base: "human-male", texture_key: "officer", shirt: [43, 71, 107], lower: [43, 71, 107], hair: [48, 33, 28], hairstyle: "short", hat: "police", costume: "suit", accessories: "badge,dutybelt", width: 1.13, height: 1.38 },
        OldLinkReceptionist => Look { base: "human-female", texture_key: "old_link_receptionist", shirt: [209, 56, 59], lower: [145, 31, 46], hair: [48, 33, 28], hairstyle: "highpony", hat: "none", costume: "dress", accessories: "namebadge,headset", width: 0.94, height: 1.38 },
        Pharmacist => Look { base: "human-male", texture_key: "pharmacist", shirt: [247, 247, 227], lower: [247, 247, 227], hair: [173, 184, 181], hairstyle: "bald", hat: "none", costume: "robe", accessories: "glasses,moustache,medicinebag", width: 1.02, height: 1.38 },
        PokefanF => Look { base: "human-female", texture_key: "pokefan_f", shirt: [33, 46, 64], lower: [33, 46, 64], hair: [76, 43, 31], hairstyle: "bun", hat: "none", costume: "dress", accessories: "apronfront,heartbadge", width: 1.31, height: 1.38 },
        PokefanM => Look { base: "human-male", texture_key: "pokefan_m", shirt: [76, 43, 31], lower: [76, 43, 31], hair: [76, 43, 31], hairstyle: "bald", hat: "none", costume: "polo", accessories: "moustache,roundglasses,ballbadge", width: 1.36, height: 1.38 },
        Pryce => Look { base: "human-male", texture_key: "pryce", shirt: [82, 176, 214], lower: [99, 64, 125], hair: [173, 184, 181], hairstyle: "sparse", hat: "none", costume: "cardigan", accessories: "cane,scarf", width: 0.98, height: 1.23 },
        Receptionist => Look { base: "human-female", texture_key: "receptionist", shirt: [43, 71, 107], lower: [43, 71, 107], hair: [76, 43, 31], hairstyle: "bob", hat: "none", costume: "suit", accessories: "skirt,namebadge,tie", width: 0.94, height: 1.38 },
        Red => Look { base: "human-male", texture_key: "red", shirt: [209, 56, 59], lower: [71, 110, 138], hair: [48, 33, 28], hairstyle: "short", hat: "redcap", costume: "vest", accessories: "backpack,whiteshirt", width: 1.00, height: 1.20 },
        RedsMom => Look { base: "human-female", texture_key: "reds_mom", shirt: [209, 56, 59], lower: [145, 31, 46], hair: [76, 43, 31], hairstyle: "bun", hat: "none", costume: "dress", accessories: "apronfront,necklace", width: 1.07, height: 1.38 },
        Rival => Look { base: "human-male", texture_key: "rival", shirt: [33, 46, 64], lower: [33, 46, 64], hair: [145, 31, 46], hairstyle: "spikes", hat: "none", costume: "jacket", accessories: "", width: 0.98, height: 1.38 },
        Rocker => Look { base: "human-male", texture_key: "rocker", shirt: [94, 156, 74], lower: [33, 46, 64], hair: [245, 125, 48], hairstyle: "mohawk", hat: "none", costume: "tee", accessories: "guitar,studdedbelt", width: 1.02, height: 1.38 },
        Rocket => Look { base: "human-male", texture_key: "rocket", shirt: [33, 46, 64], lower: [33, 46, 64], hair: [48, 33, 28], hairstyle: "short", hat: "rocket", costume: "rocket", accessories: "redr,gloves,boots", width: 1.06, height: 1.38 },
        RocketGirl => Look { base: "human-female", texture_key: "rocket_girl", shirt: [33, 46, 64], lower: [33, 46, 64], hair: [161, 59, 97], hairstyle: "long", hat: "rocket", costume: "rocket", accessories: "redr,gloves,boots,skirt", width: 0.94, height: 1.38 },
        Sabrina => Look { base: "human-female", texture_key: "sabrina", shirt: [161, 59, 97], lower: [161, 59, 97], hair: [48, 33, 28], hairstyle: "long", hat: "none", costume: "bodysuit", accessories: "shoulderguards,wristband", width: 0.94, height: 1.38 },
        Sage => Look { base: "human-male", texture_key: "sage", shirt: [204, 186, 143], lower: [204, 186, 143], hair: [173, 184, 181], hairstyle: "bald", hat: "none", costume: "robe", accessories: "beads", width: 1.09, height: 1.38 },
        Sailor => Look { base: "human-male", texture_key: "sailor", shirt: [247, 247, 227], lower: [71, 107, 186], hair: [48, 33, 28], hairstyle: "short", hat: "sailor", costume: "sailor", accessories: "neckerchief,anchor", width: 1.20, height: 1.38 },
        Scientist => Look { base: "human-male", texture_key: "scientist", shirt: [247, 247, 227], lower: [76, 43, 31], hair: [173, 184, 181], hairstyle: "short", hat: "none", costume: "coat", accessories: "glasses,pen", width: 1.00, height: 1.38 },
        StandingYoungster => Look { base: "human-male", texture_key: "standing_youngster", shirt: [71, 107, 186], lower: [43, 71, 107], hair: [48, 33, 28], hairstyle: "short", hat: "none", costume: "tee", accessories: "shorts,ballheld", width: 0.88, height: 1.05 },
        SuperNerd => Look { base: "human-male", texture_key: "super_nerd", shirt: [71, 107, 186], lower: [43, 71, 107], hair: [48, 33, 28], hairstyle: "sidepart", hat: "none", costume: "polo", accessories: "roundglasses,suspenders", width: 1.14, height: 1.38 },
        Surge => Look { base: "human-male", texture_key: "surge", shirt: [89, 122, 76], lower: [89, 122, 76], hair: [201, 156, 89], hairstyle: "flattop", hat: "none", costume: "vest", accessories: "dogtags,combatboots", width: 1.26, height: 1.38 },
        SwimmerGirl => Look { base: "human-female", texture_key: "swimmer_girl", shirt: [71, 107, 186], lower: [71, 107, 186], hair: [76, 43, 31], hairstyle: "highpony", hat: "none", costume: "swimsuit", accessories: "goggles", width: 0.94, height: 1.38 },
        SwimmerGuy => Look { base: "human-male", texture_key: "swimmer_guy", shirt: [252, 196, 145], lower: [71, 107, 186], hair: [48, 33, 28], hairstyle: "short", hat: "none", costume: "swimsuit", accessories: "goggles", width: 1.07, height: 1.38 },
        Teacher => Look { base: "human-female", texture_key: "teacher", shirt: [209, 56, 59], lower: [145, 31, 46], hair: [48, 33, 28], hairstyle: "bun", hat: "none", costume: "cardigan", accessories: "skirt", width: 1.02, height: 1.38 },
        Twin => Look { base: "human-female", texture_key: "twin", shirt: [230, 107, 145], lower: [230, 107, 145], hair: [76, 43, 31], hairstyle: "pigtails", hat: "none", costume: "dress", accessories: "ribbon", width: 0.82, height: 1.05 },
        UnusedGuy => Look { base: "human-male", texture_key: "unused_guy", shirt: [201, 156, 89], lower: [76, 43, 31], hair: [76, 43, 31], hairstyle: "sidepart", hat: "none", costume: "suit", accessories: "tealtie", width: 1.04, height: 1.38 },
        Whitney => Look { base: "human-female", texture_key: "whitney", shirt: [209, 56, 59], lower: [145, 31, 46], hair: [230, 107, 145], hairstyle: "pigtails", hat: "none", costume: "tee", accessories: "shorts,redstripe", width: 0.92, height: 1.38 },
        Will => Look { base: "human-male", texture_key: "will", shirt: [209, 56, 59], lower: [145, 31, 46], hair: [99, 64, 125], hairstyle: "sidepart", hat: "none", costume: "suit", accessories: "mask,whitegloves", width: 1.00, height: 1.38 },
        Youngster => Look { base: "human-male", texture_key: "youngster", shirt: [71, 107, 186], lower: [43, 71, 107], hair: [48, 33, 28], hairstyle: "short", hat: "none", costume: "tee", accessories: "shorts", width: 0.88, height: 1.05 },
        Outdoorsman => Look { base: "human-male", texture_key: "outdoorsman", shirt: [89, 122, 76], lower: [166, 166, 102], hair: [76, 43, 31], hairstyle: "short", hat: "hiking", costume: "vest", accessories: "backpack,beard", width: 1.18, height: 1.38 },
    }
}

/// Recolor only the documented Kenney clothing/hair UV islands. Original skin,
/// facial features, seams and shading remain in the external source texture.
pub(crate) fn texture(rgba: &[u8], size: (u32, u32), look: Look) -> Vec<u8> {
    let mut out = rgba.to_vec();
    for (i, p) in out.chunks_exact_mut(4).enumerate() {
        let x = (i as u32 % size.0) * 256 / size.0;
        let y = (i as u32 / size.0) * 256 / size.1;
        let target = if x < 155 && y >= 128 { Some(look.shirt) }
            else if x >= 156 && y >= 192 { Some(look.lower) }
            else if x < 160 && y < 125 && !(48..=118).contains(&x)
                && p[0] < 125 && p[1] < 120 && p[2] < 115 { Some(if look.hairstyle == "bald" { [239, 185, 143] } else { look.hair }) }
            else if x < 160 && y < 30 && p[0] < 125 { Some(if look.hairstyle == "bald" { [239, 185, 143] } else { look.hair }) }
            else { None };
        if let Some(target) = target {
            let shade = (0.60 + (f32::from(p[0]) + f32::from(p[1]) + f32::from(p[2])) / 765.0 * 0.40).clamp(0.60, 1.0);
            for axis in 0..3 { p[axis] = (f32::from(target[axis]) * shade).round() as u8; }
        }
    }
    out
}

pub(crate) fn head_mesh(rig: &crate::open_people::PersonRig, look: Look) -> Option<crate::mesh::SurfaceMeshData> {
    use bevy::prelude::*;
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for ((p, joints), weights) in rig.mesh.positions.iter().zip(&rig.ids).zip(&rig.weights) {
        let weight: f32 = (0..4).filter(|&i| joints[i] as usize == rig.head_joint).map(|i| weights[i]).sum();
        if weight > 0.5 { min = min.min(Vec3::from_array(*p)); max = max.max(Vec3::from_array(*p)); }
    }
    if !min.is_finite() || !max.is_finite() { return None; }
    let center = (min + max) * 0.5;
    let half = (max - min) * 0.5;
    let mut mesh = crate::mesh::SurfaceMeshData::default();
    let hair = look.hair;
    let mut orb = |p: Vec3, radius: Vec3, color: [u8; 3]| {
        ellipsoid(&mut mesh, p, radius, color);
    };
    let top = max.y;
    let width = half.x;
    match look.hairstyle {
        "long" | "bob" => {
            let length = if look.hairstyle == "long" { 0.58 } else { 0.32 };
            for side in [-1.0, 1.0] {
                orb(Vec3::new(center.x + width * side * 0.85, center.y - 0.15, center.z - 0.18),
                    Vec3::new(width * 0.30, length, half.z * 0.60), hair);
            }
        }
        "pigtails" | "rolls" => for side in [-1.0, 1.0] {
            orb(Vec3::new(center.x + width * side, center.y + 0.22, center.z - 0.15),
                Vec3::new(width * 0.46, width * 0.45, width * 0.46), hair);
        },
        "ponytail" | "highpony" | "sidepony" | "bun" | "topknot" => {
            let side = if look.hairstyle == "sidepony" { width * 0.9 } else { 0.0 };
            orb(Vec3::new(center.x + side, top - 0.13, min.z - 0.08),
                Vec3::new(width * 0.45, width * 0.42, width * 0.50), hair);
            if look.hairstyle.contains("pony") || look.hairstyle == "ponytail" {
                orb(Vec3::new(center.x + side, center.y - 0.18, min.z - 0.06),
                    Vec3::new(width * 0.31, half.y * 0.7, width * 0.36), hair);
            }
        }
        "bald" | "sparse" => for side in [-1.0, 1.0] {
            orb(Vec3::new(center.x + width * side * 0.94, center.y, center.z - 0.13),
                Vec3::new(width * 0.12, half.y * 0.35, half.z * 0.5), hair);
        },
        "mohawk" => orb(Vec3::new(center.x, top, center.z), Vec3::new(width * 0.2, width * 0.5, half.z * 0.8), hair),
        _ => {}
    }
    if look.hat != "none" {
        let color = match look.hat {
            "trainer" | "redcap" => [191, 35, 37], "bluecap" | "police" | "kris" => [32, 62, 128],
            "nurse" | "sailor" | "captain" | "reception" => [235, 234, 216],
            "straw" | "hiking" => [164, 133, 69], _ => [29, 31, 37],
        };
        let baseball = matches!(look.hat, "trainer" | "redcap" | "bluecap");
        if baseball {
            // Fit a shallow crown to the actual head instead of stacking a
            // fixed-height ball and a wide disk above the face. The bill
            // joins the crown at forehead height and leaves the eyes visible.
            orb(Vec3::new(center.x, top - half.y * 0.28, center.z - half.z * 0.04),
                Vec3::new(width * 1.01, half.y * 0.48, half.z * 0.98), color);
            orb(Vec3::new(center.x, top - half.y * 0.36, max.z - half.z * 0.12),
                Vec3::new(width * 0.78, half.y * 0.035, half.z * 0.30), color);
        } else {
            let rise = if look.hat == "top" { 0.40 } else { 0.20 };
            orb(Vec3::new(center.x, top - 0.02, center.z), Vec3::new(width * 1.03, rise, half.z * 0.88), color);
            if !matches!(look.hat, "nurse" | "kris" | "reception") {
                let wide = matches!(look.hat, "straw" | "hiking" | "top");
                orb(Vec3::new(center.x, top - 0.14, center.z + if wide { 0.0 } else { half.z * 0.75 }),
                    Vec3::new(width * if wide { 1.50 } else { 1.04 }, 0.04, half.z * if wide { 1.30 } else { 0.65 }), color);
            }
        }
        if look.hat == "nurse" {
            let badge = Vec3::new(center.x, top - 0.02, max.z - 0.03);
            orb(badge, Vec3::new(0.025, 0.075, 0.03), [183, 36, 48]);
            orb(badge, Vec3::new(0.075, 0.025, 0.03), [183, 36, 48]);
        } else if matches!(look.hat, "police" | "captain") {
            orb(Vec3::new(center.x, top - 0.02, max.z - 0.03), Vec3::new(0.075, 0.075, 0.03),
                [223, 180, 73]);
        }
    }
    if look.accessories.contains("glasses") {
        for side in [-1.0, 1.0] {
            for i in 0..24 {
                let angle = i as f32 * std::f32::consts::TAU / 24.0;
                orb(Vec3::new(center.x + side * width * 0.42 + angle.cos() * width * 0.29,
                    center.y + half.y * 0.04 + angle.sin() * half.y * 0.19, max.z + 0.016),
                    Vec3::splat(0.020), [43, 39, 37]);
            }
        }
    }
    if look.accessories.contains("moustache") || look.accessories.contains("beard") {
        orb(Vec3::new(center.x, center.y - half.y * 0.43, max.z - 0.01),
            Vec3::new(width * 0.42, half.y * if look.accessories.contains("beard") { 0.35 } else { 0.09 }, 0.055), hair);
    }
    if mesh.positions.is_empty() { return None; }
    let bind = rig.inverse_binds[rig.head_joint];
    let normal = bind.inverse().transpose();
    for (p, n) in mesh.positions.iter_mut().zip(&mut mesh.normals) {
        *p = bind.transform_point3(Vec3::from_array(*p)).to_array();
        *n = normal.transform_vector3(Vec3::from_array(*n)).normalize().to_array();
    }
    Some(mesh)
}

fn ellipsoid(mesh: &mut crate::mesh::SurfaceMeshData, center: bevy::prelude::Vec3,
    radius: bevy::prelude::Vec3, rgb: [u8; 3]) {
    use bevy::prelude::*;
    const SIDES: usize = 16;
    const ROWS: usize = 8;
    let first = mesh.positions.len() as u32;
    let linear = |v: u8| { let s = f32::from(v) / 255.0; if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) } };
    let color = [linear(rgb[0]), linear(rgb[1]), linear(rgb[2]), 1.0];
    for row in 0..=ROWS {
        let latitude = std::f32::consts::PI * row as f32 / ROWS as f32;
        for side in 0..=SIDES {
            let longitude = std::f32::consts::TAU * side as f32 / SIDES as f32;
            let direction = Vec3::new(latitude.sin() * longitude.cos(), latitude.cos(), latitude.sin() * longitude.sin());
            mesh.positions.push((center + direction * radius).to_array());
            mesh.normals.push((direction / radius).normalize().to_array());
            mesh.colors.push(color); mesh.uvs.push([0.0; 2]);
        }
    }
    for row in 0..ROWS { for side in 0..SIDES {
        let a = first + (row * (SIDES + 1) + side) as u32; let b = a + (SIDES + 1) as u32;
        mesh.indices.extend([a, b, a + 1, a + 1, b, b + 1]);
    } }
}

pub(crate) fn costume_mesh(rig: &crate::open_people::PersonRig, look: Look) -> Option<crate::mesh::SurfaceMeshData> {
    use bevy::prelude::*;
    let skirt = matches!(look.costume, "dress" | "kimono" | "robe") || look.accessories.contains("skirt");
    let coat = look.costume == "coat";
    if !skirt && !coat { return None; }
    let mut mesh = crate::mesh::SurfaceMeshData::default();
    // A hem follows the source pelvis. Legs retain the downloaded weighted
    // rig and authored clips beneath it, rather than reverting to rigid limbs.
    let top = 1.55;
    let bottom = if look.costume == "robe" || look.costume == "kimono" { 0.36 }
        else if coat { 0.92 } else { 0.77 };
    let color = if coat { look.shirt } else { look.lower };
    let first = mesh.positions.len() as u32;
    let rgb = color.map(|v| { let s = f32::from(v)/255.0; if s <= 0.04045 { s/12.92 } else { ((s+0.055)/1.055).powf(2.4) } });
    const SIDES: usize = 32;
    for (y, width, depth) in [(top, 0.36, 0.31), (bottom, if coat { 0.43 } else { 0.52 }, 0.37)] {
        for i in 0..=SIDES {
            let angle = i as f32 * std::f32::consts::TAU / SIDES as f32;
            mesh.positions.push([angle.cos()*width, y, angle.sin()*depth + 0.05]);
            mesh.normals.push(Vec3::new(angle.cos()/width, 0.10, angle.sin()/depth).normalize().to_array());
            mesh.uvs.push([0.0;2]); mesh.colors.push([rgb[0],rgb[1],rgb[2],1.0]);
        }
    }
    for i in 0..SIDES {
        let a = first+i as u32; let b=a+(SIDES+1) as u32;
        mesh.indices.extend([a,b,a+1,a+1,b,b+1]);
    }
    let bind = rig.inverse_binds[rig.hips_joint];
    let normal = bind.inverse().transpose();
    for (p,n) in mesh.positions.iter_mut().zip(&mut mesh.normals) {
        *p=bind.transform_point3(Vec3::from_array(*p)).to_array();
        *n=normal.transform_vector3(Vec3::from_array(*n)).normalize().to_array();
    }
    Some(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clothing_recolor_keeps_the_face_and_bare_arm_islands() {
        let original = vec![180; 256 * 256 * 4];
        let result = texture(&original, (256,256), look(CharacterKind::Nurse));
        for (x,y) in [(80,60),(225,80)] {
            let at=(y*256+x)*4;
            assert_eq!(&result[at..at+4], &original[at..at+4]);
        }
        let at=(150*256+80)*4;
        assert_ne!(&result[at..at+3], &original[at..at+3]);
        assert_eq!(result[at+3], original[at+3]);
    }
    #[test]
    fn recognizable_staff_and_children_have_explicit_costumes_and_scale() {
        assert_eq!(look(CharacterKind::Nurse).hat, "nurse");
        assert_eq!(look(CharacterKind::Officer).hat, "police");
        assert_eq!(look(CharacterKind::Rocket).hat, "rocket");
        assert_eq!(look(CharacterKind::Scientist).costume, "coat");
        assert!(look(CharacterKind::Youngster).height < look(CharacterKind::Officer).height);
        assert!(look(CharacterKind::PokefanM).width > look(CharacterKind::Youngster).width);
    }
}
