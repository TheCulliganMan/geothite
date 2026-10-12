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
        BugCatcher => Look { base: "human-male", texture_key: "bug_catcher", shirt: [71, 107, 186], lower: [166, 166, 102], hair: [76, 43, 31], hairstyle: "short", hat: "straw", costume: "tee", accessories: "shorts,net", width: 0.98, height: 1.05 },
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
        StandingYoungster => Look { base: "human-male", texture_key: "standing_youngster", shirt: [71, 107, 186], lower: [43, 71, 107], hair: [48, 33, 28], hairstyle: "short", hat: "none", costume: "tee", accessories: "shorts,ballheld", width: 0.98, height: 1.05 },
        SuperNerd => Look { base: "human-male", texture_key: "super_nerd", shirt: [71, 107, 186], lower: [43, 71, 107], hair: [48, 33, 28], hairstyle: "sidepart", hat: "none", costume: "polo", accessories: "roundglasses,suspenders", width: 1.14, height: 1.38 },
        Surge => Look { base: "human-male", texture_key: "surge", shirt: [89, 122, 76], lower: [89, 122, 76], hair: [201, 156, 89], hairstyle: "flattop", hat: "none", costume: "vest", accessories: "dogtags,combatboots", width: 1.26, height: 1.38 },
        SwimmerGirl => Look { base: "human-female", texture_key: "swimmer_girl", shirt: [71, 107, 186], lower: [71, 107, 186], hair: [76, 43, 31], hairstyle: "highpony", hat: "none", costume: "swimsuit", accessories: "goggles", width: 0.94, height: 1.38 },
        SwimmerGuy => Look { base: "human-male", texture_key: "swimmer_guy", shirt: [252, 196, 145], lower: [71, 107, 186], hair: [48, 33, 28], hairstyle: "short", hat: "none", costume: "swimsuit", accessories: "goggles,shorts", width: 1.07, height: 1.38 },
        Teacher => Look { base: "human-female", texture_key: "teacher", shirt: [209, 56, 59], lower: [145, 31, 46], hair: [48, 33, 28], hairstyle: "bun", hat: "none", costume: "cardigan", accessories: "skirt", width: 1.02, height: 1.38 },
        Twin => Look { base: "human-female", texture_key: "twin", shirt: [230, 107, 145], lower: [230, 107, 145], hair: [76, 43, 31], hairstyle: "pigtails", hat: "none", costume: "dress", accessories: "ribbon", width: 0.82, height: 1.05 },
        UnusedGuy => Look { base: "human-male", texture_key: "unused_guy", shirt: [201, 156, 89], lower: [76, 43, 31], hair: [76, 43, 31], hairstyle: "sidepart", hat: "none", costume: "suit", accessories: "tealtie", width: 1.04, height: 1.38 },
        Whitney => Look { base: "human-female", texture_key: "whitney", shirt: [209, 56, 59], lower: [145, 31, 46], hair: [230, 107, 145], hairstyle: "pigtails", hat: "none", costume: "tee", accessories: "shorts,redstripe", width: 0.92, height: 1.38 },
        Will => Look { base: "human-male", texture_key: "will", shirt: [209, 56, 59], lower: [145, 31, 46], hair: [99, 64, 125], hairstyle: "sidepart", hat: "none", costume: "suit", accessories: "mask,whitegloves", width: 1.00, height: 1.38 },
        Youngster => Look { base: "human-male", texture_key: "youngster", shirt: [71, 107, 186], lower: [43, 71, 107], hair: [48, 33, 28], hairstyle: "short", hat: "none", costume: "tee", accessories: "shorts", width: 0.98, height: 1.05 },
        Outdoorsman => Look { base: "human-male", texture_key: "outdoorsman", shirt: [89, 122, 76], lower: [166, 166, 102], hair: [76, 43, 31], hairstyle: "short", hat: "hiking", costume: "vest", accessories: "backpack,beard", width: 1.18, height: 1.38 },
    }
}

/// Recolor only the documented Kenney clothing/hair UV islands. Original skin,
/// facial features, seams and shading remain in the external source texture.
#[cfg(test)]
pub(crate) fn texture(rgba: &[u8], size: (u32, u32), look: Look) -> Vec<u8> {
    let mut out = rgba.to_vec();
    for (i, p) in out.chunks_exact_mut(4).enumerate() {
        let x = (i as u32 % size.0) * 256 / size.0;
        let y = (i as u32 / size.0) * 256 / size.1;
        let target = if x < 155 && y >= 128 { Some(look.shirt) }
            else if x >= 156 && y >= 192 { Some(look.lower) }
            else if x < 160 && y < 125 && !(if matches!(look.hairstyle,"bald"|"sparse") {60..=102} else {48..=118}).contains(&x)
                && p[0] < 125 && p[1] < 120 && p[2] < 115 { Some(if look.hairstyle == "bald" && !(32..68).contains(&y) { [239, 185, 143] } else { look.hair }) }
            // Female forehead hair reaches row 47, with temple locks through
            // row 55 outside the central face. Keep the brows and ear interiors
            // out of this mask rather than extending a rectangle over them.
            else if x < 160 && (y < 30 || (look.base=="human-female" && y<48)
                || (look.base=="human-female" && y<56 && (x<60 || x>=100))
                // The male forelock stops above the brows; its sideburns
                // continue down beside, rather than over, the ear interiors.
                || (look.base=="human-male" && !matches!(look.hairstyle,"bald"|"sparse")
                    && (y<43 || (y<51 && (x<61 || x>=96))
                        || (y<56 && (x<57 || x>=100))
                        || (y<62 && ((48..57).contains(&x) || (100..106).contains(&x) || x>=115))))
                || (y < 44 && matches!(look.hairstyle,"bald"|"sparse")))
                && p[0] < 125 && p[1] < 120 && p[2] < 115 { Some(if look.hairstyle == "bald" { [239, 185, 143] } else { look.hair }) }
            else { None };
        if let Some(target) = target {
            let shade = if look.hairstyle=="bald" && target==[239,185,143] {1.0} else {
                (0.60 + (f32::from(p[0]) + f32::from(p[1]) + f32::from(p[2])) / 765.0 * 0.40).clamp(0.60, 1.0)
            };
            for axis in 0..3 { p[axis] = (f32::from(target[axis]) * shade).round() as u8; }
        }
    }
    out
}

/// The four unpainted boys must not inherit the adult source's beard shadow.
/// Both shared atlases use the same face island. Borrow its clean cheek/chin
/// painting, keeping the original eyes, nose and lip pixels on the male rig.
#[cfg(test)]
pub(crate) fn clean_child_face(out: &mut [u8], size: (u32,u32), look: Look,
    clean: &[u8], clean_size: (u32,u32)) {
    if !matches!(look.texture_key,"bugsy"|"bug_catcher"|"gameboy_kid"|"standing_youngster") {return;}
    for (i,p) in out.chunks_exact_mut(4).enumerate() {
        let x=(i as u32%size.0)*256/size.0;let y=(i as u32/size.0)*256/size.1;
        if !(51..108).contains(&x) || !(63..99).contains(&y) {continue;}
        // Preserve the nostrils and lips; replace their surrounding stubble.
        if ((74..84).contains(&x) && y<65)
            || ((70..88).contains(&x) && (65..71).contains(&y)
                && f32::from(p[0])>f32::from(p[1])*1.35
                && f32::from(p[0])>f32::from(p[2])*1.2) {continue;}
        let smooth=|v:f32| {let t=v.clamp(0.0,1.0);t*t*(3.0-2.0*t)};
        let blend=smooth((x-51).min(107-x) as f32/4.0)
            *smooth((y-63).min(98-y) as f32/4.0);
        let cx=x*clean_size.0/256;let cy=y*clean_size.1/256;
        let at=((cy*clean_size.0+cx)*4) as usize;
        for a in 0..3 {p[a]=(f32::from(p[a])*(1.0-blend)+f32::from(clean[at+a])*blend).round() as u8;}
    }
}

/// Match the already supplied child painting's readable eyes and expression.
/// Borrow only the central face; each appearance keeps its hair and ears.
#[cfg(test)]
pub(crate) fn paint_child_face(out: &mut [u8], size: (u32,u32), look: Look,
    painting: &[u8], painting_size: (u32,u32)) {
    if !matches!(look.texture_key,"bugsy"|"bug_catcher"|"gameboy_kid"|"standing_youngster") {return;}
    let mut target=[0.0;3];let mut source=[0.0;3];let mut count=0.0;
    // Match the painting to the existing cheek color, avoiding a warm central
    // patch surrounded by the cooler source skin at temples and jaw edges.
    for x in [65,66,67,92,93,94] {for y in 77..85 {
        let at=((y*size.1/256)*size.0+x*size.0/256) as usize*4;
        let from=((y*painting_size.1/256)*painting_size.0+x*painting_size.0/256) as usize*4;
        for a in 0..3 {target[a]+=f32::from(out[at+a]);source[a]+=f32::from(painting[from+a]);}
        count+=1.0;
    }}
    let tone=std::array::from_fn::<_,3,_>(|a|(target[a]/count)/(source[a]/count).max(1.0));
    for (i,p) in out.chunks_exact_mut(4).enumerate() {
        let x=(i as u32%size.0) as f32*256.0/size.0 as f32;
        let y=(i as u32/size.0) as f32*256.0/size.1 as f32;
        if !(54.0..106.0).contains(&x) || !(44.0..101.0).contains(&y) {continue;}
        if y<50.0 && p[0]<150 && p[1]<120 && p[2]<150 {continue;}
        let smooth=|t:f32| {let t=t.clamp(0.0,1.0);t*t*(3.0-2.0*t)};
        let blend=smooth((x-54.0).min(106.0-x)/5.0)*smooth((y-44.0).min(101.0-y)/5.0);
        // Sample in normalized atlas coordinates, regardless of painting size.
        let sx=x*painting_size.0 as f32/256.0;let sy=y*painting_size.1 as f32/256.0;
        let ix=sx.floor() as u32;let iy=sy.floor() as u32;
        let sample=|a:usize,dx:u32,dy:u32| {
            f32::from(painting[(((iy+dy).min(painting_size.1-1)*painting_size.0
                +(ix+dx).min(painting_size.0-1))*4) as usize+a])
        };
        for a in 0..3 {
            let top=sample(a,0,0)*(1.0-sx.fract())+sample(a,1,0)*sx.fract();
            let bottom=sample(a,0,1)*(1.0-sx.fract())+sample(a,1,1)*sx.fract();
            let color=(top*(1.0-sy.fract())+bottom*sy.fract())*tone[a];
            p[a]=(f32::from(p[a])*(1.0-blend)+color*blend).round() as u8;
        }
    }
}

/// Shorter, tapered jaws on the four boys using the shared adult rig. Refine
/// the head before reshaping so curved jaw edges do not remain long facets.
pub(crate) fn child_head(rig: &crate::open_people::PersonRig,
    mesh: &mut crate::mesh::SurfaceMeshData, ids: &mut Vec<[u16;4]>, weights: &mut Vec<[f32;4]>, look: Look) {
    use bevy::prelude::*;
    use std::collections::HashMap;
    if !matches!(look.texture_key,"bugsy"|"bug_catcher"|"gameboy_kid"|"standing_youngster") {return;}
    let is_head=|i:usize,ids:&[[u16;4]],weights:&[[f32;4]]|
        (0..4).filter(|&j| ids[i][j] as usize==rig.head_joint).map(|j|weights[i][j]).sum::<f32>()>0.5;
    let mut min=Vec3::splat(f32::INFINITY);let mut max=Vec3::splat(f32::NEG_INFINITY);
    for (i,p) in mesh.positions.iter().enumerate() {if is_head(i,ids,weights) {
        min=min.min(Vec3::from_array(*p));max=max.max(Vec3::from_array(*p));
    }}
    if !min.is_finite() || !max.is_finite() {return;}
    let center=(min+max)*0.5;let half=(max-min)*0.5;
    for _ in 0..2 {
        let mut edges=HashMap::new();let old=std::mem::take(&mut mesh.indices);
        for face in old.chunks_exact(3) {
            let [a,b,c]=[face[0],face[1],face[2]];
            if ![a,b,c].iter().all(|&i|is_head(i as usize,ids,weights)) {
                mesh.indices.extend([a,b,c]);continue;
            }
            let mut midpoint=|a:u32,b:u32| -> u32 {
                *edges.entry((a.min(b),a.max(b))).or_insert_with(|| {
                    let (a,b)=(a as usize,b as usize);let index=mesh.positions.len() as u32;
                    mesh.positions.push(((Vec3::from_array(mesh.positions[a])+Vec3::from_array(mesh.positions[b]))*0.5).to_array());
                    mesh.normals.push((Vec3::from_array(mesh.normals[a])+Vec3::from_array(mesh.normals[b])).normalize().to_array());
                    mesh.colors.push(std::array::from_fn(|j|(mesh.colors[a][j]+mesh.colors[b][j])*0.5));
                    mesh.uvs.push(std::array::from_fn(|j|(mesh.uvs[a][j]+mesh.uvs[b][j])*0.5));
                    let mut combined=HashMap::<u16,f32>::new();
                    for i in [a,b] {for j in 0..4 {*combined.entry(ids[i][j]).or_default()+=weights[i][j]*0.5;}}
                    let mut joints:Vec<_>=combined.into_iter().filter(|(_,w)|*w>0.0).collect();
                    joints.sort_by(|a,b|b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));joints.truncate(4);
                    let sum=joints.iter().map(|(_,w)|w).sum::<f32>();
                    ids.push(std::array::from_fn(|j|joints.get(j).map_or(0,|p|p.0)));
                    weights.push(std::array::from_fn(|j|joints.get(j).map_or(0.0,|p|p.1/sum)));
                    index
                })
            };
            let ab=midpoint(a,b);let bc=midpoint(b,c);let ca=midpoint(c,a);
            mesh.indices.extend([a,ab,ca,ab,b,bc,ca,bc,c,ab,bc,ca]);
        }
    }
    let lower=|y:f32| {let t=((center.y+half.y*0.10-y)/(half.y*1.10)).clamp(0.0,1.0);t*t*(3.0-2.0*t)};
    for i in 0..mesh.positions.len() {if is_head(i,ids,weights) {
        let mut p=Vec3::from_array(mesh.positions[i]);let t=lower(p.y);
        p.x=center.x+(p.x-center.x)*(1.0-0.18*t);
        p.z=center.z+(p.z-center.z)*(1.0-0.08*t);
        p.y+=half.y*0.07*t;mesh.positions[i]=p.to_array();
    }}
    // Weld only normal accumulation across UV seams. UVs and skin attributes
    // remain independent, so facial features keep their existing placement.
    let key=|p:[f32;3]|p.map(|v|(v*100000.0).round() as i32);
    let mut normals=HashMap::<[i32;3],Vec3>::new();
    for face in mesh.indices.chunks_exact(3) {
        if !face.iter().all(|&i|is_head(i as usize,ids,weights)) {continue;}
        let p=[face[0],face[1],face[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
        let n=(p[1]-p[0]).cross(p[2]-p[0]);
        for &i in face {*normals.entry(key(mesh.positions[i as usize])).or_default()+=n;}
    }
    for i in 0..mesh.positions.len() {if is_head(i,ids,weights) {
        if let Some(n)=normals.get(&key(mesh.positions[i])) {
            if n.length_squared()>1e-12 {
                let original=Vec3::from_array(mesh.normals[i]);
                mesh.normals[i]=original.lerp(n.normalize(),lower(mesh.positions[i][1]).sqrt()).normalize().to_array();
            }
        }
    }}
}

/// Soften the downward normal band on the shared lower face. The painted
/// facial features and silhouette remain authored by the source rig.
pub(crate) fn soften_jaw_normals(rig: &crate::open_people::PersonRig,
    mesh: &mut crate::mesh::SurfaceMeshData, ids: &[[u16;4]], weights: &[[f32;4]]) {
    use bevy::prelude::*;
    let head=|i:usize| (0..4).filter(|&j|ids[i][j] as usize==rig.head_joint)
        .map(|j|weights[i][j]).sum::<f32>()>0.5;
    let mut min=f32::INFINITY;let mut max=f32::NEG_INFINITY;
    for (i,p) in mesh.positions.iter().enumerate() {if head(i) {min=min.min(p[1]);max=max.max(p[1]);}}
    if !min.is_finite() || max<=min {return;}
    for i in 0..mesh.positions.len() {
        if !head(i) {continue;}
        let n=Vec3::from_array(mesh.normals[i]);
        let radial=Vec3::new(n.x,0.0,n.z).normalize_or_zero();
        if n.y>=-0.30 || radial.length_squared()<0.5 {continue;}
        let t=((min+(max-min)*0.43-mesh.positions[i][1])/((max-min)*0.24)).clamp(0.0,1.0);
        if t==0.0 {continue;}
        let t=t*t*(3.0-2.0*t);
        let softened=(radial+Vec3::Y* -0.28).normalize();
        mesh.normals[i]=n.lerp(softened,t*0.85).normalize().to_array();
    }
}

/// Split the shared trouser surface at a mid-thigh hem. Bare legs reuse the
/// existing arm skin island, including painted atlases, without changing shoes.
pub(crate) fn shorts_body(rig: &crate::open_people::PersonRig,
    mesh: &mut crate::mesh::SurfaceMeshData, joints: &mut Vec<[u16;4]>,
    weights: &mut Vec<[f32;4]>, look: Look) {
    use bevy::prelude::*;
    let swimsuit = look.texture_key == "swimmer_girl";
    if !swimsuit && !look.accessories.split(',').any(|a| a=="shorts") {return;}
    let thigh=rig.inverse_binds[rig.leg_joints[0][0]].inverse().transform_point3(Vec3::ZERO).y;
    let knee=rig.inverse_binds[rig.leg_joints[0][1]].inverse().transform_point3(Vec3::ZERO).y;
    // Swimwear rises at the outer hip while keeping the center covered.
    // The same signed distance drives both clipping and the fitted lip,
    // keeping the garment and skin cuts coincident through animation.
    let hem_at = |p: [f32;3]| if swimsuit {
        knee+(thigh-knee)*0.82+p[0].abs()*0.35
    } else { knee+(thigh-knee)*0.47 };
    // Texture islands duplicate coincident vertices. Use one normal across
    // those seams so curved edges and the raised fabric lip stay joined.
    let key=|p:[f32;3]|p.map(|v|(v*100_000.0).round() as i32);
    let mut seam_normals=std::collections::HashMap::<[i32;3],Vec3>::new();
    for i in 0..mesh.positions.len() {
        if mesh.uvs[i][0]>=0.605 && mesh.uvs[i][1]>=0.745 {
            *seam_normals.entry(key(mesh.positions[i])).or_default()+=Vec3::from_array(mesh.normals[i]);
        }
    }
    for i in 0..mesh.positions.len() {
        if mesh.uvs[i][0]>=0.605 && mesh.uvs[i][1]>=0.745 {
            mesh.normals[i]=seam_normals[&key(mesh.positions[i])].normalize().to_array();
        }
    }
    // Curved edge midpoints soften exposed shins and knees while retaining
    // every original anchor. Taper the curvature away at shoes and the waist.
    for _ in 0..2 {
        let faces=std::mem::take(&mut mesh.indices);
        let mut midpoints=std::collections::HashMap::new();
        for face in faces.chunks_exact(3) {
            let [a,b,c]=[face[0] as usize,face[1] as usize,face[2] as usize];
            if ![a,b,c].iter().all(|&i|mesh.uvs[i][0]>=0.605 && mesh.uvs[i][1]>=0.745) {
                mesh.indices.extend_from_slice(face);continue;
            }
            let mut midpoint=|a:usize,b:usize| *midpoints.entry((a.min(b),a.max(b))).or_insert_with(|| {
                let index=mesh.positions.len();let pa=Vec3::from_array(mesh.positions[a]);
                let pb=Vec3::from_array(mesh.positions[b]);let na=Vec3::from_array(mesh.normals[a]);
                let nb=Vec3::from_array(mesh.normals[b]);let center=(pa+pb)*0.5;
                let fade=((center.y-0.30)/0.15).clamp(0.0,1.0)
                    *((hem_at(center.to_array())+0.16-center.y)/0.16).clamp(0.0,1.0);
                let curve=-((pb-pa).dot(na)*na+(pa-pb).dot(nb)*nb)*0.125*fade;
                mesh.positions.push((center+curve).to_array());
                mesh.normals.push((na+nb).normalize().to_array());
                mesh.uvs.push(((Vec2::from_array(mesh.uvs[a])+Vec2::from_array(mesh.uvs[b]))*0.5).to_array());
                mesh.colors.push(((Vec4::from_array(mesh.colors[a])+Vec4::from_array(mesh.colors[b]))*0.5).to_array());
                let mut influences=std::collections::BTreeMap::<u16,f32>::new();
                for i in [a,b] {for j in 0..4 {
                    *influences.entry(joints[i][j]).or_default()+=weights[i][j]*0.5;
                }}
                let mut influences=influences.into_iter().collect::<Vec<_>>();
                influences.sort_by(|a,b|b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                influences.truncate(4);let total=influences.iter().map(|x|x.1).sum::<f32>();
                let mut js=[0;4];let mut ws=[0.0;4];
                for (j,(id,w)) in influences.into_iter().enumerate() {js[j]=id;ws[j]=w/total;}
                joints.push(js);weights.push(ws);index as u32
            });
            let ab=midpoint(a,b);let bc=midpoint(b,c);let ca=midpoint(c,a);
            mesh.indices.extend([a as u32,ab,ca,ab,b as u32,bc,ca,bc,c as u32,ab,bc,ca]);
        }
    }
    let faces=std::mem::take(&mut mesh.indices);
    // Insert cut vertices once per edge, so both sides share exactly the same
    // position and skinning. UV seams are duplicated only after clipping.
    let mut cuts=std::collections::HashMap::new();
    for face in faces.chunks_exact(3) {
        let ids=[face[0] as usize,face[1] as usize,face[2] as usize];
        if !ids.iter().all(|&i| mesh.uvs[i][0]>=0.605 && mesh.uvs[i][1]>=0.745) {
            mesh.indices.extend_from_slice(face);continue;
        }
        for bare in [false,true] {
            let mut polygon=Vec::new();
            for edge in 0..3 {
                let a=ids[edge];let b=ids[(edge+1)%3];
                let ay=mesh.positions[a][1]-hem_at(mesh.positions[a]);
                let by=mesh.positions[b][1]-hem_at(mesh.positions[b]);
                let inside=|distance:f32| if bare {distance<=0.0} else {distance>=0.0};
                if inside(ay) {polygon.push(a);}
                if inside(ay)!=inside(by) {
                    let key=(a.min(b),a.max(b));
                    let cut=*cuts.entry(key).or_insert_with(|| {
                        let t=ay/(ay-by);let index=mesh.positions.len();
                        let mix=|x:[f32;3],y:[f32;3]|Vec3::from_array(x).lerp(Vec3::from_array(y),t);
                        mesh.positions.push(mix(mesh.positions[a],mesh.positions[b]).to_array());
                        mesh.normals.push(mix(mesh.normals[a],mesh.normals[b]).normalize().to_array());
                        mesh.uvs.push(Vec2::from_array(mesh.uvs[a]).lerp(Vec2::from_array(mesh.uvs[b]),t).to_array());
                        mesh.colors.push(Vec4::from_array(mesh.colors[a]).lerp(Vec4::from_array(mesh.colors[b]),t).to_array());
                        let mut influences=std::collections::BTreeMap::<u16,f32>::new();
                        for (i,factor) in [(a,1.0-t),(b,t)] {for j in 0..4 {
                            *influences.entry(joints[i][j]).or_default()+=weights[i][j]*factor;
                        }}
                        let mut influences=influences.into_iter().collect::<Vec<_>>();
                        influences.sort_by(|a,b|b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                        influences.truncate(4);let total=influences.iter().map(|x|x.1).sum::<f32>();
                        let mut js=[0;4];let mut ws=[0.0;4];
                        for (j,(id,w)) in influences.into_iter().enumerate() {js[j]=id;ws[j]=w/total;}
                        joints.push(js);weights.push(ws);index
                    });
                    polygon.push(cut);
                }
            }
            if !bare {
                let edge=polygon.iter().copied().filter(|&i|(mesh.positions[i][1]-hem_at(mesh.positions[i])).abs()<1e-6)
                    .collect::<Vec<_>>();
                // A narrow fitted fabric lip; keep the skin below on its
                // original surface and close the underside between them.
                let mut raised=std::collections::HashMap::new();
                for i in &mut polygon {
                    let original=*i;let y=mesh.positions[original][1];
                    let fade=(1.0-(y-hem_at(mesh.positions[original]))/0.08).clamp(0.0,1.0).powi(2);
                    if fade==0.0 {continue;}
                    let n=Vec3::from_array(mesh.normals[original]);
                    let outward=Vec3::new(n.x,0.0,n.z).normalize_or_zero();
                    *i=mesh.positions.len();
                    mesh.positions.push((Vec3::from_array(mesh.positions[original])+outward*0.012*fade).to_array());
                    mesh.normals.push(mesh.normals[original]);mesh.uvs.push(mesh.uvs[original]);
                    mesh.colors.push(mesh.colors[original]);joints.push(joints[original]);weights.push(weights[original]);
                    raised.insert(original,*i);
                }
                if edge.len()==2 && edge[0]!=edge[1] {
                    let a=edge[0];let b=edge[1];
                    let outer=[raised[&a],raised[&b]];
                    let start=mesh.positions.len() as u32;
                    for original in [a,b,outer[1],outer[0]] {
                        mesh.positions.push(mesh.positions[original]);
                        mesh.normals.push(Vec3::new(if swimsuit {mesh.positions[original][0].signum()*0.35} else {0.0},-1.0,0.0).normalize().to_array());
                        mesh.uvs.push(mesh.uvs[original]);mesh.colors.push(mesh.colors[original]);
                        joints.push(joints[original]);weights.push(weights[original]);
                    }
                    for tri in [[start,start+1,start+2],[start,start+2,start+3]] {
                        let p=tri.map(|i|Vec3::from_array(mesh.positions[i as usize]));
                        if (p[1]-p[0]).cross(p[2]-p[0]).length_squared()<1e-18 {continue;}
                        let tri=if (p[1]-p[0]).cross(p[2]-p[0]).y>0.0 {[tri[0],tri[2],tri[1]]} else {tri};
                        mesh.indices.extend(tri);
                    }
                }
            }
            if bare {for i in &mut polygon {
                let original=*i;*i=mesh.positions.len();
                mesh.positions.push(mesh.positions[original]);mesh.normals.push(mesh.normals[original]);
                mesh.colors.push(mesh.colors[original]);joints.push(joints[original]);weights.push(weights[original]);
                // An interior arm sample avoids hand creases and atlas boundaries.
                mesh.uvs.push([0.90+(mesh.uvs[original][0]-0.8)*0.12,
                    0.19+(hem_at(mesh.positions[original])-mesh.positions[original][1])*0.16]);
            }}
            for i in 1..polygon.len().saturating_sub(1) {
                let tri=[polygon[0] as u32,polygon[i] as u32,polygon[i+1] as u32];
                let p=tri.map(|j|Vec3::from_array(mesh.positions[j as usize]));
                if (p[1]-p[0]).cross(p[2]-p[0]).length_squared()>1e-18 {mesh.indices.extend(tri);}
            }
        }
    }
    if swimsuit {
        for (&p, uv) in mesh.positions.iter().zip(&mut mesh.uvs) {
            if uv[0]>=0.605 && uv[1]>=0.745 {
                *uv=swimsuit_uv(p);
            }
        }
    }
}

/// Use one continuous blue fabric patch across the swimsuit's waist seam.
pub(crate) fn swimsuit_uv(p: [f32;3]) -> [f32;2] {
    [0.82+p[0]*0.06,0.85+(p[1]-1.4)*0.01]
}

/// Painted shirts share a front/back atlas island. Split rear torso faces
/// into plain fabric patches so collars, buttons and zippers stay on the front.
pub(crate) fn correct_body_uvs(mesh: &mut crate::mesh::SurfaceMeshData, look: Look,
    joints: &mut Vec<[u16;4]>, weights: &mut Vec<[f32;4]>) {
    let patch = match look.texture_key {
        "chris" => [0.16, 0.60, 0.05, 0.24],
        "lass" => [48.0/256.0, 170.0/256.0, 0.025, 0.12],
        "cooltrainer_f" => [48.0/256.0, 172.0/256.0, 0.025, 0.11],
        "nurse" => [47.0/256.0, 174.0/256.0, 0.020, 0.08],
        "gramps" => [46.0/256.0, 166.0/256.0, 0.025, 0.10],
        "super_nerd" => [92.0/256.0, 174.0/256.0, 0.025, 0.11],
        _ => return,
    };
    // Split rear faces at the atlas boundary. Reassigning only some corners
    // makes a triangle interpolate through the front zipper and collar.
    // The T-pose forearms can share this atlas island; the mesh-space torso
    // width also excludes them so bare arms and sleeve cuffs stay intact.
    for face in mesh.indices.chunks_exact_mut(3) {
        let ids=[face[0] as usize,face[1] as usize,face[2] as usize];
        let rear=ids.iter().map(|&i| mesh.positions[i][2]).sum::<f32>()/3.0;
        let height=ids.iter().map(|&i| mesh.positions[i][1]).sum::<f32>()/3.0;
        if rear>=0.03 || !(1.25..2.55).contains(&height)
            || !ids.iter().all(|&i| mesh.positions[i][0].abs()<0.4 && mesh.positions[i][2]<=0.1
                && mesh.uvs[i][0]<if look.texture_key=="chris" {0.625} else {0.5}
                && mesh.uvs[i][1]>0.47) { continue; }
        for (index,&i) in face.iter_mut().zip(&ids) {
            let p=mesh.positions[i];
            *index=mesh.positions.len() as u32;
            mesh.positions.push(p);mesh.normals.push(mesh.normals[i]);
            mesh.colors.push(mesh.colors[i]);
            mesh.uvs.push([patch[0]+p[0]*patch[2],patch[1]+(2.55-p[1])*patch[3]]);
            joints.push(joints[i]);weights.push(weights[i]);
        }
    }
}

/// Soft rectangular bags, pockets and straps are original renderer geometry.
/// Attach to Chest so the pack stays with the torso as the legs and arms swing.
pub(crate) fn backpack_mesh(rig: &crate::open_people::PersonRig, look: Look)
    -> Option<crate::mesh::SurfaceMeshData> {
    use bevy::prelude::*;
    // The existing chest-attached accessory slot also carries fitted beads.
    if look.accessories.split(',').any(|a|a=="beads") {return crate::beads::mesh(rig,look);}
    if look.accessories.split(',').any(|a|a=="apricornbasket") {return Some(crate::apricorn_basket::mesh(rig));}
    if !look.accessories.split(',').any(|a| a=="backpack" || a=="hikingpack") { return None; }
    let mut mesh = crate::mesh::SurfaceMeshData::default();
    let (fabric, trim) = match look.texture_key {
        "chris" => ([211,170,62],[97,73,37]),
        "kris" => ([226,174,58],[115,81,40]),
        "red" => ([151,102,55],[74,49,32]),
        "cal" => ([54,101,153],[31,57,87]),
        _ => ([97,119,61],[54,67,39]),
    };
    let hiking = matches!(look.texture_key,"outdoorsman"|"brock");
    let tall = if hiking {0.43} else {0.34};
    soft_box(&mut mesh,Vec3::new(0.0,1.91,-0.29),Vec3::new(0.31,tall,0.17),fabric);
    soft_box(&mut mesh,Vec3::new(0.0,1.77,-0.46),Vec3::new(0.23,0.15,0.055),fabric);
    // A recessed dark zipper and a small pull give the outer pocket scale.
    soft_box(&mut mesh,Vec3::new(0.0,1.90,-0.515),Vec3::new(0.20,0.012,0.009),trim);
    soft_box(&mut mesh,Vec3::new(0.16,1.875,-0.53),Vec3::new(0.014,0.038,0.01),[211,201,160]);
    for side in [-1.0,1.0] {
        let x=side*0.25;
        // Fitted shoulder loop follows the sloping chest rather than a circle
        // floating ahead of it. The return passes through the back panel.
        let path=[Vec3::new(x,1.65,0.25),Vec3::new(x,1.83,0.25),
            Vec3::new(x,2.04,0.225),Vec3::new(x,2.24,0.195),
            Vec3::new(x,2.49,0.13),Vec3::new(x,2.535,0.01),
            Vec3::new(x,2.48,-0.145),Vec3::new(x,2.15,-0.20),
            Vec3::new(x,1.84,-0.20),Vec3::new(x,1.65,-0.12)];
        let sample=|row:usize| {
            let segment=(row/8)%path.len();let t=(row%8) as f32/8.0;
            let p0=path[(segment+path.len()-1)%path.len()];let p1=path[segment];
            let p2=path[(segment+1)%path.len()];let p3=path[(segment+2)%path.len()];
            ((p1*2.0)+(p2-p0)*t+(p0*2.0-p1*5.0+p2*4.0-p3)*t*t
                +(-p0+p1*3.0-p2*3.0+p3)*t*t*t)*0.5
        };
        let first=mesh.positions.len() as u32;
        let rows=path.len()*8;
        const SIDES: usize=8;
        for row in 0..=rows {
            let center=sample(row);
            let tangent=(sample((row+1)%rows)-sample((row+rows-1)%rows)).normalize();
            let radial=Vec3::X.cross(tangent).normalize();
            for i in 0..=SIDES {
                let a=std::f32::consts::TAU*i as f32/SIDES as f32;
                let offset=Vec3::X*(a.cos()*0.036)+radial*(a.sin()*0.012);
                mesh.positions.push((center+offset).to_array());
                mesh.normals.push((Vec3::X*(a.cos()/0.036)+radial*(a.sin()/0.012)).normalize().to_array());
                mesh.colors.push(linear_color(trim));mesh.uvs.push([0.0;2]);
            }
        }
        for row in 0..rows { for i in 0..SIDES {
            let a=first+(row*(SIDES+1)+i) as u32;let b=a+(SIDES+1) as u32;
            mesh.indices.extend([a,b,a+1,a+1,b,b+1]);
        } }
    }
    let bind=rig.inverse_binds[rig.chest_joint];
    let normal=bind.inverse().transpose();
    for (p,n) in mesh.positions.iter_mut().zip(&mut mesh.normals) {
        *p=bind.transform_point3(Vec3::from_array(*p)).to_array();
        *n=normal.transform_vector3(Vec3::from_array(*n)).normalize().to_array();
    }
    Some(mesh)
}

fn linear_color(rgb: [u8;3]) -> [f32;4] {
    let c=rgb.map(|v| { let s=f32::from(v)/255.0;
        if s<=0.04045 {s/12.92} else {((s+0.055)/1.055).powf(2.4)} });
    [c[0],c[1],c[2],1.0]
}

/// Superellipsoid with rounded rectangular cross-sections and smooth normals.
fn soft_box(mesh: &mut crate::mesh::SurfaceMeshData, center: bevy::prelude::Vec3,
    radius: bevy::prelude::Vec3, rgb: [u8;3]) {
    use bevy::prelude::*;
    const ROWS: usize=20;const SIDES: usize=40;
    let first=mesh.positions.len() as u32;
    let power=|v:f32| v.signum()*v.abs().sqrt();
    for row in 0..=ROWS {
        let a=std::f32::consts::PI*row as f32/ROWS as f32;
        for i in 0..=SIDES {
            let b=std::f32::consts::TAU*i as f32/SIDES as f32;
            let d=Vec3::new(power(a.sin()*b.cos()),power(a.cos()),power(a.sin()*b.sin()));
            let n=Vec3::new(d.x.powi(3),d.y.powi(3),d.z.powi(3))/radius;
            mesh.positions.push((center+d*radius).to_array());
            mesh.normals.push(n.normalize().to_array());
            mesh.colors.push(linear_color(rgb));mesh.uvs.push([0.0;2]);
        }
    }
    for row in 0..ROWS { for i in 0..SIDES {
        let a=first+(row*(SIDES+1)+i) as u32;let b=a+(SIDES+1) as u32;
        mesh.indices.extend([a,b,a+1,a+1,b,b+1]);
    } }
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
    let top = max.y;
    let width = half.x;
    if matches!(look.hairstyle, "long" | "bob") {
        let painted_hair = crate::npc_textures::get(look.texture_key).map(|painting| {
            let mut sum = [0_u32; 3];
            let mut count = 0;
            for y in painting.size.1/25..painting.size.1/10 {
                for x in painting.size.0/20..painting.size.0/8 {
                    let at = ((y*painting.size.0+x)*4) as usize;
                    for a in 0..3 { sum[a] += u32::from(painting.rgba[at+a]); }
                    count += 1;
                }
            }
            sum.map(|s| (s/count) as u8)
        }).unwrap_or(hair);
        hair_curtain(&mut mesh, center, half, look.hairstyle == "long", painted_hair,
            crate::npc_textures::get(look.texture_key));
    }
    if matches!(look.hairstyle,"pigtails"|"rolls"|"ponytail"|"highpony"|"sidepony"|"bun"|"topknot") {
        tied_hair(&mut mesh,center,half,look);
    }
    match look.hairstyle {
        "long" | "bob" => {}
        "pigtails" | "rolls" | "ponytail" | "highpony" | "sidepony" | "bun" | "topknot" => {}
        "bald" | "sparse" => {}
        "mohawk" => ellipsoid(&mut mesh, Vec3::new(center.x, top, center.z), Vec3::new(width * 0.2, width * 0.5, half.z * 0.8), hair),
        _ => {}
    }
    if look.hat != "none" && !crate::uniform_caps::append(&mut mesh,center,half,look.hat) {
        let color = match look.hat {
            "trainer" | "redcap" => [191, 35, 37], "bluecap" | "police" | "kris" => [32, 62, 128],
            "sailor" | "captain" | "reception" => [235, 234, 216],
            "straw" | "hiking" => [164, 133, 69], _ => [29, 31, 37],
        };
        let baseball = matches!(look.hat, "trainer" | "redcap" | "bluecap");
        if baseball {
            // Fit a shallow crown to the actual head instead of stacking a
            // fixed-height ball and a wide disk above the face. The bill
            // joins the crown at forehead height and leaves the eyes visible.
            ellipsoid(&mut mesh, Vec3::new(center.x, top - half.y * 0.28, center.z - half.z * 0.04),
                Vec3::new(width * 1.01, half.y * 0.48, half.z * 0.98), color);
            ellipsoid(&mut mesh, Vec3::new(center.x, top - half.y * 0.36, max.z - half.z * 0.12),
                Vec3::new(width * 0.78, half.y * 0.035, half.z * 0.30), color);
        } else {
            let rise = if look.hat == "top" { 0.40 } else { 0.20 };
            ellipsoid(&mut mesh, Vec3::new(center.x, top - 0.02, center.z), Vec3::new(width * 1.03, rise, half.z * 0.88), color);
            if !matches!(look.hat, "kris" | "reception") {
                let wide = matches!(look.hat, "straw" | "hiking" | "top");
                ellipsoid(&mut mesh, Vec3::new(center.x, top - 0.14, center.z + if wide { 0.0 } else { half.z * 0.75 }),
                    Vec3::new(width * if wide { 1.50 } else { 1.04 }, 0.04, half.z * if wide { 1.30 } else { 0.65 }), color);
            }
        }
        if matches!(look.hat, "police" | "captain") {
            ellipsoid(&mut mesh, Vec3::new(center.x, top - 0.02, max.z - 0.03), Vec3::new(0.075, 0.075, 0.03),
                [223, 180, 73]);
        }
    }
    if look.accessories.contains("glasses") {
        fitted_glasses(&mut mesh, rig, center, half, look);
    }
    if look.accessories.contains("moustache") {
        crate::moustache::append(&mut mesh,rig,center,half,look.hair);
    }
    if look.accessories.contains("beard") {
        fitted_beard(&mut mesh,rig,center,half,look);
    }
    if look.texture_key == "erika" && look.accessories.contains("flower") {
        crate::hair_flower::append(&mut mesh,center,half);
    }
    if matches!(look.texture_key, "morty" | "black_belt") && look.accessories.contains("headband") {
        crate::headband::append(&mut mesh,rig,center,half,look);
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

/// Hair gathers start inside the scalp and continue into one tapered surface.
/// Distinct control curves replace the disconnected ball-and-oval attachments.
fn tied_hair(mesh: &mut crate::mesh::SurfaceMeshData, center: bevy::prelude::Vec3,
    half: bevy::prelude::Vec3, look: Look) {
    use bevy::prelude::*;
    let local=|x:f32,y:f32,z:f32| center+half*Vec3::new(x,y,z);
    // Granny's painting has warm gray hair; the previous cool gray orb looked
    // like a separate accessory even where it met the existing hair surface.
    let mut hair=if look.texture_key=="granny" {[147,137,130]} else {look.hair};
    if matches!(look.texture_key, "whitney" | "misty" | "janine" | "kris") {
        if let Some(painting) = crate::npc_textures::get(look.texture_key) {
            // Match the gathers to the painted scalp rather than the
            // brighter fallback swatch. This strip contains hair only.
            let (w,h) = painting.size;
            let mut sum = [0_u32;3];
            let mut count = 0;
            for y in h/25..h/10 { for x in w/20..w/8 {
                let at = ((y*w+x)*4) as usize;
                for a in 0..3 {sum[a] += u32::from(painting.rgba[at+a]);}
                count += 1;
            }}
            hair = sum.map(|c| (c/count) as u8);
        }
    }
    match look.hairstyle {
        "pigtails"|"rolls" => for side in [-1.0,1.0] {
            let roll=look.hairstyle=="rolls";
            let kris=look.texture_key=="kris";
            if kris {
                // Gather at the scalp, then fan into overlapping tapered
                // locks. The former broad oval read as a rigid leaf.
                hair_lock(mesh, [local(side*0.73,0.40,-0.38),
                    local(side*1.17,0.48,-0.53), local(side*1.48,-0.14,-0.65),
                    local(side*1.39,-0.76,-0.60)],
                    half.x*0.19,half.x*0.22,0.75,false,hair);
                hair_lock(mesh, [local(side*1.20,0.02,-0.60),
                    local(side*1.42,-0.18,-0.63), local(side*1.57,-0.35,-0.64),
                    local(side*1.60,-0.56,-0.60)],
                    half.x*0.14,half.x*0.06,0.75,false,hair);
                hair_lock(mesh, [local(side*1.09,0.11,-0.62),
                    local(side*1.14,-0.16,-0.70), local(side*1.20,-0.45,-0.73),
                    local(side*1.17,-0.70,-0.69)],
                    half.x*0.12,half.x*0.055,0.75,false,hair);
                continue;
            }
            let curve=if roll {
                [local(side*0.76,0.30,-0.40),local(side*1.13,0.33,-0.60),
                    local(side*1.30,-0.15,-0.73),local(side*0.94,-0.43,-0.70)]
            } else {
                [local(side*0.73,0.40,-0.38),local(side*1.25,0.50,-0.59),
                    local(side*1.30,-0.19,-0.67),
                    local(side*1.10,-0.76,-0.70)]
            };
            hair_lock(mesh,curve,half.x*0.23,half.x*0.25,0.80,false,hair);
        },
        "bun"|"topknot" => {
            let topknot=look.hairstyle=="topknot";
            let curve=if topknot {
                [local(0.0,0.82,-0.22),local(0.0,1.03,-0.26),
                    local(0.0,1.20,-0.34),local(0.0,1.28,-0.43)]
            } else {
                [local(0.0,0.52,-0.72),local(0.0,0.55,-0.95),
                    local(0.0,0.47,-1.20),local(0.0,0.34,-1.48)]
            };
            hair_lock(mesh,curve,half.x*0.22,half.x*0.32,0.86,true,hair);
        },
        "ponytail"|"highpony"|"sidepony" => {
            let high=look.hairstyle=="highpony";
            let side=look.hairstyle=="sidepony";
            let curve=if side {
                [local(0.67,0.55,-0.46),local(1.11,0.65,-0.77),
                    local(1.48,0.03,-0.95),local(1.19,-0.54,-0.87)]
            } else {
                let y=if high {0.74} else {0.32};
                [local(0.0,y,-0.75),local(0.0,y+0.15,-1.38),
                    local(0.18,-0.45,-1.58),local(0.14,if high {-1.02} else {-1.16},-1.12)]
            };
            hair_lock(mesh,curve,half.x*0.20,half.x*if side {0.31} else {0.34},0.82,false,hair);
        },
        _=>{}
    }
}

/// Cubic centerline with a rounded, tapered elliptical cross-section. Small
/// longitudinal grooves give gathered hair direction without separate spikes.
fn hair_lock(mesh: &mut crate::mesh::SurfaceMeshData, controls: [bevy::prelude::Vec3;4],
    root_radius: f32, fullness: f32, flatten: f32, round_tip: bool, rgb: [u8;3]) {
    use bevy::prelude::*;
    const ROWS:usize=40;const SIDES:usize=40;
    let center=|t:f32| {let s=1.0-t;
        controls[0]*s.powi(3)+controls[1]*(3.0*s*s*t)+controls[2]*(3.0*s*t*t)+controls[3]*t.powi(3)};
    let point=|t:f32,a:f32| {
        let tangent=(center((t+0.001).min(1.0))-center((t-0.001).max(0.0))).normalize();
        let u=(Vec3::X-tangent*tangent.x).normalize();let v=tangent.cross(u).normalize();
        let radius=root_radius*(1.0-t)+fullness*(std::f32::consts::PI*t).sin().max(0.0)
            .powf(if round_tip {0.5} else {0.8})+0.006*t;
        let groove=1.0-0.027*(a*8.0+t*1.5).cos();
        center(t)+(u*a.cos()+v*(a.sin()*flatten))*(radius*groove)
    };
    let first=mesh.positions.len() as u32;
    for row in 0..=ROWS {for col in 0..=SIDES {
        let t=row as f32/ROWS as f32;let a=std::f32::consts::TAU*col as f32/SIDES as f32;
        let dt=point((t+0.001).min(1.0),a)-point((t-0.001).max(0.0),a);
        let da=point(t,a+0.001)-point(t,a-0.001);
        mesh.positions.push(point(t,a).to_array());mesh.normals.push(da.cross(dt).normalize().to_array());
        mesh.colors.push(linear_color(rgb));mesh.uvs.push([0.0;2]);
    }}
    for row in 0..ROWS {for col in 0..SIDES {
        let a=first+(row*(SIDES+1)+col) as u32;let b=a+(SIDES+1) as u32;
        mesh.indices.extend([a,a+1,b,a+1,b+1,b]);
    }}
    for (row,reverse) in [(0,true),(ROWS,false)] {
        let t=row as f32/ROWS as f32;
        let tangent=(center((t+0.001).min(1.0))-center((t-0.001).max(0.0))).normalize();
        let pole=mesh.positions.len() as u32;
        mesh.positions.push(center(t).to_array());mesh.normals.push((tangent*if reverse {-1.0} else {1.0}).to_array());
        mesh.colors.push(linear_color(rgb));mesh.uvs.push([0.0;2]);
        for col in 0..SIDES {
            let a=first+(row*(SIDES+1)+col) as u32;
            if reverse {mesh.indices.extend([pole,a+1,a]);} else {mesh.indices.extend([pole,a,a+1]);}
        }
    }
}

/// Continuous wire frames in mesh bind space. The eye plane comes from the
/// supplied face, rather than the nose's foremost vertex; arms return to ears.
fn fitted_glasses(mesh: &mut crate::mesh::SurfaceMeshData, rig: &crate::open_people::PersonRig,
    center: bevy::prelude::Vec3, half: bevy::prelude::Vec3, look: Look) {
    use bevy::prelude::*;
    let eye_y=center.y+half.y*0.04;
    let face_depth=|x:f32| {
        let mut depth=f32::NEG_INFINITY;
        for face in rig.mesh.indices.chunks_exact(3) {
            let [a,b,c]=[face[0],face[1],face[2]].map(|i| Vec3::from_array(rig.mesh.positions[i as usize]));
            if [a,b,c].iter().any(|p| p.y<center.y-half.y) {continue;}
            let d=(b.y-c.y)*(a.x-c.x)+(c.x-b.x)*(a.y-c.y);
            if d.abs()<1e-7 {continue;}
            let u=((b.y-c.y)*(x-c.x)+(c.x-b.x)*(eye_y-c.y))/d;
            let v=((c.y-a.y)*(x-c.x)+(a.x-c.x)*(eye_y-c.y))/d;
            if u>=0.0 && v>=0.0 && u+v<=1.0 {depth=depth.max(a.z*u+b.z*v+c.z*(1.0-u-v));}
        }
        if depth.is_finite() {depth} else {center.z+half.z*0.9}
    };
    let front=face_depth(center.x+half.x*0.42)+0.045;
    let radius=half.x*0.027;
    let rgb=[43,39,37];
    let round=look.accessories.contains("roundglasses");
    for side in [-1.0,1.0] {
        let mut rim=Vec::new();
        for i in 0..=64 {
            let angle=std::f32::consts::TAU*i as f32/64.0;
            let shape=|v:f32| if round {v} else {v.signum()*v.abs().powf(0.72)};
            let x=center.x+side*half.x*0.42+shape(angle.cos())*half.x*0.29;
            let y=eye_y+shape(angle.sin())*half.y*if round {0.19} else {0.15};
            rim.push(Vec3::new(x,y,front-0.035*((x-center.x)/half.x).powi(2)));
        }
        wire_frame(mesh,&rim,true,radius,rgb);
        let outer=Vec3::new(center.x+side*half.x*0.71,eye_y,front-0.035*0.71f32.powi(2));
        let controls=[outer,Vec3::new(center.x+side*half.x*1.02,eye_y+half.y*0.03,front-0.08),
            Vec3::new(center.x+side*half.x*1.02,eye_y+half.y*0.06,center.z-half.z*0.06),
            Vec3::new(center.x+side*half.x*0.94,eye_y-half.y*0.12,center.z-half.z*0.25)];
        let arm:Vec<_>=(0..=32).map(|i| {
            let t=i as f32/32.0;let s=1.0-t;
            controls[0]*s.powi(3)+controls[1]*(3.0*s*s*t)+controls[2]*(3.0*s*t*t)+controls[3]*t.powi(3)
        }).collect();
        wire_frame(mesh,&arm,false,radius,rgb);
    }
    let bridge:Vec<_>=(0..=20).map(|i| {
        let t=i as f32/20.0;
        Vec3::new(center.x+half.x*0.26*(t-0.5),eye_y+half.y*0.05*(std::f32::consts::PI*t).sin(),
            front+half.z*0.10*(std::f32::consts::PI*t).sin())
    }).collect();
    wire_frame(mesh,&bridge,false,radius,rgb);
}

fn wire_frame(mesh: &mut crate::mesh::SurfaceMeshData, path: &[bevy::prelude::Vec3],
    closed: bool, radius: f32, rgb: [u8;3]) {
    use bevy::prelude::*;
    const SIDES:usize=10;
    let first=mesh.positions.len() as u32;
    let last=path.len()-1;
    let mut previous_u: Option<Vec3>=None;
    for (i,&p) in path.iter().enumerate() {
        let before=if i==0 {if closed {last-1} else {0}} else {i-1};
        let after=if i==last {if closed {1} else {last}} else {i+1};
        let tangent=(path[after]-path[before]).normalize();
        let reference=if tangent.dot(Vec3::Z).abs()>0.9 {Vec3::Y} else {Vec3::Z};
        let u=previous_u.map(|u| (u-tangent*u.dot(tangent)).normalize())
            .unwrap_or_else(|| tangent.cross(reference).normalize());
        previous_u=Some(u);
        let v=tangent.cross(u).normalize();
        for j in 0..=SIDES {
            let angle=std::f32::consts::TAU*j as f32/SIDES as f32;
            let n=u*angle.cos()+v*angle.sin();
            mesh.positions.push((p+n*radius).to_array());mesh.normals.push(n.to_array());
            mesh.colors.push(linear_color(rgb));mesh.uvs.push([0.0;2]);
        }
    }
    for row in 0..last {for j in 0..SIDES {
        let a=first+(row*(SIDES+1)+j) as u32;let b=a+(SIDES+1) as u32;
        mesh.indices.extend([a,a+1,b,a+1,b+1,b]);
    }}
}

/// Fit facial hair to the source jaw surface. The upper edge dips below the
/// mouth centrally and rises at the cheeks; it never covers the lips or nose.
pub(crate) fn head_surface_depth(rig: &crate::open_people::PersonRig,
    center: bevy::prelude::Vec3, half: bevy::prelude::Vec3, x:f32, y:f32) -> f32 {
    use bevy::prelude::*;
        let mut front=f32::NEG_INFINITY;
        let mut nearest=(f32::INFINITY,center.z);
        for face in rig.mesh.indices.chunks_exact(3) {
            let a=Vec3::from_array(rig.mesh.positions[face[0] as usize]);
            let b=Vec3::from_array(rig.mesh.positions[face[1] as usize]);
            let c=Vec3::from_array(rig.mesh.positions[face[2] as usize]);
            if a.y<center.y-half.y*1.05 || b.y<center.y-half.y*1.05 || c.y<center.y-half.y*1.05 {continue;}
            let d=(b.y-c.y)*(a.x-c.x)+(c.x-b.x)*(a.y-c.y);
            if d.abs()<1e-7 {continue;}
            let u=((b.y-c.y)*(x-c.x)+(c.x-b.x)*(y-c.y))/d;
            let v=((c.y-a.y)*(x-c.x)+(a.x-c.x)*(y-c.y))/d;
            if u>=0.0 && v>=0.0 && u+v<=1.0 {front=front.max(a.z*u+b.z*v+c.z*(1.0-u-v));}
            if a.z.max(b.z).max(c.z)<center.z {continue;}
            for (p,q) in [(a,b),(b,c),(c,a)] {
                let edge=Vec2::new(q.x-p.x,q.y-p.y);
                let t=if edge.length_squared()>1e-8 {
                    (Vec2::new(x-p.x,y-p.y).dot(edge)/edge.length_squared()).clamp(0.0,1.0)
                } else {0.0};
                let hit=p.lerp(q,t);let distance=Vec2::new(x-hit.x,y-hit.y).length_squared();
                if distance<nearest.0-1e-7 || ((distance-nearest.0).abs()<1e-7 && hit.z>nearest.1) {
                    nearest=(distance,hit.z);
                }
            }
        }
        if front.is_finite() {front} else {nearest.1}
}

fn fitted_beard(mesh: &mut crate::mesh::SurfaceMeshData, rig: &crate::open_people::PersonRig,
    center: bevy::prelude::Vec3, half: bevy::prelude::Vec3, look: Look) {
    use bevy::prelude::*;
    let elder=look.texture_key=="elder";
    let surface=|x:f32,y:f32| head_surface_depth(rig,center,half,x,y);
    let point=|u:f32,v:f32| {
        let side=u*2.0-1.0;
        let x=center.x+side*half.x*(0.84-v*if elder {0.48} else {0.35});
        let upper=center.y-half.y*(0.65-0.42*side.abs().powf(1.4));
        let lower=center.y-half.y*(if elder {1.08} else {0.96}-0.18*side.abs());
        let y=upper+(lower-upper)*v;
        Vec3::new(x,y,surface(x,y)+0.022+0.018*(std::f32::consts::PI*v).sin())
    };
    const COLS: usize=32;const ROWS: usize=12;
    let first=mesh.positions.len() as u32;
    for row in 0..=ROWS {for col in 0..=COLS {
        let u=col as f32/COLS as f32;let v=row as f32/ROWS as f32;
        let du=point((u+0.002).min(1.0),v)-point((u-0.002).max(0.0),v);
        let dv=point(u,(v+0.002).min(1.0))-point(u,(v-0.002).max(0.0));
        mesh.positions.push(point(u,v).to_array());
        mesh.normals.push(dv.cross(du).normalize_or_zero().to_array());
        mesh.colors.push(linear_color(look.hair));mesh.uvs.push([0.0;2]);
    }}
    for row in 0..ROWS {for col in 0..COLS {
        let a=first+(row*(COLS+1)+col) as u32;let b=a+(COLS+1) as u32;
        mesh.indices.extend([a,b,a+1,a+1,b,b+1]);
    }}
}

/// A rounded rear curtain starts inside the scalp, flows over the head and
/// turns inward at the hem. Curved cheek openings avoid vertical side slabs.
fn hair_curtain(mesh: &mut crate::mesh::SurfaceMeshData, center: bevy::prelude::Vec3,
    half: bevy::prelude::Vec3, long: bool, rgb: [u8; 3],
    painting: Option<&crate::npc_textures::Texture>) {
    use bevy::prelude::*;
    const ROWS: usize = 56;
    const SIDES: usize = 64;
    let bottom=if long {-1.42} else {-0.76};
    // Root radius is deliberately inside the scalp. The old exposed top rim
    // made a step in the silhouette and a light stripe across the back.
    let profiles=[Vec3::new(0.88,0.36,0.38),Vec3::new(0.66,0.74,0.77),
        Vec3::new(0.40,0.98,1.02),Vec3::new(0.10,1.10,1.10),
        Vec3::new(-0.35,1.10,1.08),Vec3::new(bottom+0.17,if long {0.97} else {1.05},0.98),
        Vec3::new(bottom+0.06,if long {0.89} else {0.99},0.91),
        Vec3::new(bottom,if long {0.85} else {0.94},0.86)];
    let point=|t:f32,u:f32,inner:bool| {
        let q=t.clamp(0.0,1.0)*(profiles.len()-1) as f32;
        let i=(q.floor() as usize).min(profiles.len()-2);let f=q-i as f32;
        let p1=profiles[i];let p2=profiles[i+1];
        let p0=if i==0 {p1-(p2-p1)} else {profiles[i-1]};
        let p3=if i+2==profiles.len() {p2+(p2-p1)} else {profiles[i+2]};
        // Cubic Hermite with limited component slopes prevents overshoot at
        // the closely spaced hem rings and keeps the vertical ordering.
        let limited=|a:f32,b:f32| if a*b<=0.0 {0.0} else {a.signum()*((a.abs()+b.abs())*0.5).min(2.0*a.abs().min(b.abs()))};
        let slope=|a:Vec3,b:Vec3| Vec3::new(limited(a.x,b.x),limited(a.y,b.y),limited(a.z,b.z));
        let m1=slope(p1-p0,p2-p1);let m2=slope(p2-p1,p3-p2);
        let p=p1*(2.0*f.powi(3)-3.0*f*f+1.0)+m1*(f.powi(3)-2.0*f*f+f)
            +p2*(-2.0*f.powi(3)+3.0*f*f)+m2*(f.powi(3)-f*f);
        let opening=std::f32::consts::PI*(0.34+0.075*t*t*(3.0-2.0*t));
        let angle=opening+(std::f32::consts::TAU-2.0*opening)*u;
        let (s,c)=angle.sin_cos();
        let gather=(std::f32::consts::PI*t).sin().powi(2);
        // Broad flowing locks and a gently varied, inward-turning hem avoid
        // the old cylindrical slab with an almost perfectly horizontal end.
        let lock=(angle*9.0+0.35*t+0.25*(angle*3.0).sin()).cos();
        let end=t.powi(8);
        let taper=1.0-end*(0.025+0.035*(1.0-lock)*0.5);
        let ridge=(1.0+0.025*gather*lock)*taper;
        let thickness=if inner {0.026} else {0.0};
        let w=half.x*p.y*ridge-thickness;let d=half.z*p.z*ridge-thickness;
        let tips=half.y*(if long {0.16} else {0.07})*end
            *(0.55+0.45*(angle*7.0+0.3*(angle*3.0).sin()).cos());
        center+Vec3::new(w*s,half.y*p.x+tips,d*c-0.025)
    };
    let first=mesh.positions.len() as u32;
    let layer=(ROWS+1)*(SIDES+1);
    for inner in [false,true] {for row in 0..=ROWS {for col in 0..=SIDES {
        let t=row as f32/ROWS as f32;let u=col as f32/SIDES as f32;
        let p=point(t,u,inner);
        let across=point(t,(u+0.0005).min(1.0),inner)-point(t,(u-0.0005).max(0.0),inner);
        let down=point((t+0.0005).min(1.0),u,inner)-point((t-0.0005).max(0.0),u,inner);
        let n=down.cross(across).normalize()*if inner {-1.0} else {1.0};
        // Match the source atlas's shaded hair rather than using the full
        // palette color, which made the added surface a much lighter panel.
        let strand=(u*std::f32::consts::TAU*9.0+0.8*t).cos();
        let shade=0.68+0.08*t+0.045*strand*(std::f32::consts::PI*t).sin();
        let color=if let Some(painting)=painting {
            // The atlas's left hair strip stays outside the face and ear.
            // Filter a small footprint into vertex albedo so the painted locks
            // continue onto the separately modeled curtain, without sampling
            // skin or applying a second dark palette tint to the painting.
            let (w,h)=painting.size;
            // Hold the atlas row nearly steady down the lock: traversing the
            // painted horizontal waves vertically created stacked hair bands.
            let bend=(std::f32::consts::PI*t).sin();
            let x=((0.012+0.11*u+0.004*bend*(u*std::f32::consts::TAU).cos())*w as f32) as i32;
            let y=((0.045+0.010*bend)*h as f32) as i32;
            let mut sum=[0_u32;3];
            for dy in -2..=2 {for dx in -2..=2 {
                let px=(x+dx).clamp(0,w as i32-1) as u32;
                let py=(y+dy).clamp(0,h as i32-1) as u32;
                let at=((py*w+px)*4) as usize;
                for a in 0..3 {sum[a]+=u32::from(painting.rgba[at+a]);}
            }}
            linear_color(sum.map(|s| (s as f32/25.0*0.96).round() as u8))
        } else {
            linear_color(rgb.map(|c| (f32::from(c)*shade).round() as u8))
        };
        mesh.positions.push(p.to_array());mesh.normals.push(n.to_array());
        mesh.colors.push(color);mesh.uvs.push([0.0;2]);
    }}}
    let mut quad=|a:usize,b:usize,c:usize,d:usize| {
        mesh.indices.extend([first+a as u32,first+b as u32,first+c as u32,
            first+a as u32,first+c as u32,first+d as u32]);
    };
    for row in 0..ROWS {for col in 0..SIDES {
        let a=row*(SIDES+1)+col;let b=a+SIDES+1;
        quad(a,b,b+1,a+1);quad(a+layer,a+layer+1,b+layer+1,b+layer);
    }}
    for col in 0..SIDES {
        quad(col,col+layer,col+layer+1,col+1);
        let a=ROWS*(SIDES+1)+col;quad(a,a+1,a+layer+1,a+layer);
    }
    for row in 0..ROWS {
        let a=row*(SIDES+1);let b=a+SIDES+1;
        quad(a,b,b+layer,a+layer);
        quad(a+SIDES,a+SIDES+layer,b+SIDES+layer,b+SIDES);
    }
}

/// Smooth elliptical profiles with a thin inner shell and closed rims. The
/// slope of each authored ring contributes to the normals, including the hem.
fn profile_shell(mesh: &mut crate::mesh::SurfaceMeshData, x: f32, z: f32,
    profiles: &[(f32, f32, f32)], arc: (f32, f32), thickness: f32, rgb: [u8; 3]) {
    use bevy::prelude::*;
    const SIDES: usize = 40;
    let first = mesh.positions.len() as u32;
    let color = rgb.map(|v| { let s = f32::from(v) / 255.0;
        if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) } });
    let rows = profiles.len();
    let layer = rows * (SIDES + 1);
    for inner in [false, true] {
        for (j, &(y, width, depth)) in profiles.iter().enumerate() {
            let above = profiles[j.saturating_sub(1)];
            let below = profiles[(j + 1).min(rows - 1)];
            for i in 0..=SIDES {
                let a = arc.0 + (arc.1 - arc.0) * i as f32 / SIDES as f32;
                let (s, c) = a.sin_cos();
                let w = width - if inner { thickness } else { 0.0 };
                let d = depth - if inner { thickness } else { 0.0 };
                let tangent = Vec3::new(w * c, 0.0, -d * s);
                let vertical = Vec3::new((above.1 - below.1) * s, above.0 - below.0,
                    (above.2 - below.2) * c);
                let n = tangent.cross(vertical).normalize() * if inner { -1.0 } else { 1.0 };
                mesh.positions.push([x + w * s, y, z + d * c]);
                mesh.normals.push(n.to_array());
                mesh.colors.push([color[0], color[1], color[2], 1.0]);
                mesh.uvs.push([0.0; 2]);
            }
        }
    }
    let mut quad = |a: usize, b: usize, c: usize, d: usize| {
        mesh.indices.extend([first+a as u32, first+b as u32, first+c as u32,
            first+a as u32, first+c as u32, first+d as u32]);
    };
    for j in 0..rows-1 { for i in 0..SIDES {
        let a = j*(SIDES+1)+i; let b = a+SIDES+1;
        quad(a,b,b+1,a+1);
        quad(a+layer,a+layer+1,b+layer+1,b+layer);
    } }
    for i in 0..SIDES {
        quad(i,i+1,i+layer+1,i+layer);
        let a = (rows-1)*(SIDES+1)+i;
        quad(a,a+layer,a+layer+1,a+1);
    }
    if arc.1-arc.0 < std::f32::consts::TAU-0.001 {
        for j in 0..rows-1 {
            let a=j*(SIDES+1); let b=a+SIDES+1;
            quad(a,a+layer,b+layer,b);
            quad(a+SIDES,b+SIDES,b+SIDES+layer,a+SIDES+layer);
        }
    }
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

pub(crate) struct CostumeMesh {
    pub surface: crate::mesh::SurfaceMeshData,
    pub joints: Vec<[u16; 4]>,
    pub weights: Vec<[f32; 4]>,
}

pub(crate) fn costume_mesh(rig: &crate::open_people::PersonRig, look: Look) -> Option<CostumeMesh> {
    use bevy::prelude::*;
    if let Some(cape) = crate::cape::build(rig,look) { return Some(cape); }
    if let Some(uniform) = crate::uniform_details::build(rig,look) {return Some(uniform);}
    let skirt = matches!(look.costume, "dress" | "kimono" | "robe") || look.accessories.contains("skirt");
    let coat = look.costume == "coat";
    if !skirt && !coat { return None; }
    let mut mesh = crate::mesh::SurfaceMeshData::default();
    // Garments share the body skin palette and bind space. The fitted waist
    // follows the torso surface; lower fabric follows both legs with a center blend.
    let top = 1.62;
    let bottom = if coat {0.92} else {crate::garment_body::hem(look)?};
    let mut color = if coat { look.shirt } else { look.lower };
    if matches!(look.texture_key, "erika" | "receptionist" | "nurse" | "falkner" | "elm" | "oak" | "scientist" | "blaine" | "pharmacist" | "elder" | "sage" | "kurt" | "kurt_outside" | "kimono_girl") {
        if let Some(painting) = crate::npc_textures::get(look.texture_key) {
            // Match the separate skirt to its plain painted fabric,
            // rather than retaining the old palette swatch.
            let (w,h) = painting.size;
            let plain_u = if matches!(look.texture_key, "falkner" | "elm" | "oak" | "scientist" | "blaine" | "pharmacist" | "elder" | "sage" | "kurt" | "kurt_outside" | "kimono_girl") {0.075} else {0.24};
            let x = (w as f32*plain_u) as i32;
            let y = (h as f32*0.75) as i32;
            let mut sum = [0_u32;3];
            for dy in -2..=2 { for dx in -2..=2 {
                let px = (x+dx).clamp(0,w as i32-1) as u32;
                let py = (y+dy).clamp(0,h as i32-1) as u32;
                let at = ((py*w+px)*4) as usize;
                for a in 0..3 { sum[a] += u32::from(painting.rgba[at+a]); }
            }}
            color = sum.map(|c| (c/25) as u8);
        }
    }
    let hem = if coat { 0.43 } else { 0.52 };
    let profiles: [(f32,f32,f32);7] = [
        (top + 0.05, 0.36, 0.32),
        (top, 0.365, 0.325),
        (top - 0.10, 0.38, 0.335),
        ((top + bottom)*0.5, (0.37+hem)*0.5, 0.35),
        (bottom + 0.045, hem, 0.37),
        (bottom + 0.015, hem-0.006, 0.364),
        (bottom, hem-0.022, 0.348),
    ];
    let arc = if coat { (std::f32::consts::PI*0.13,std::f32::consts::PI*1.87) }
        else { (0.0,std::f32::consts::TAU) };
    // Skinning cannot bend a single large quad around a knee. Keep the exact
    // neutral profile, with enough vertical sections to carry its weight field.
    let mut sections = Vec::new();
    for pair in profiles.windows(2) {
        let steps = ((pair[0].0-pair[1].0).abs()/0.045).ceil() as usize;
        for i in 0..steps {
            let t = i as f32/steps as f32;
            sections.push((pair[0].0+(pair[1].0-pair[0].0)*t,
                pair[0].1+(pair[1].1-pair[0].1)*t,
                pair[0].2+(pair[1].2-pair[0].2)*t));
        }
    }
    sections.push(*profiles.last().unwrap());
    profile_shell(&mut mesh, 0.0, 0.05, &sections, arc, 0.018, color);
    let seam_weights = crate::garment_fit::fit(&mut mesh, rig);
    crate::robe_drape::finish(&mut mesh, look, bottom);
    if look.texture_key == "elm" {
        // Fitting changes the waist slope. Retaining profile-shell normals
        // leaves a dark angular patch across the upper coat during Run.
        mesh.normals.fill([0.;3]);
        for face in mesh.indices.chunks_exact(3) {
            let p = [face[0],face[1],face[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
            let n = (p[1]-p[0]).cross(p[2]-p[0]);
            for &i in face { let old=Vec3::from_array(mesh.normals[i as usize]);mesh.normals[i as usize]=(old+n).to_array(); }
        }
        for n in &mut mesh.normals { *n=Vec3::from_array(*n).normalize().to_array(); }
    }
    let thigh = rig.leg_joints.map(|j| rig.inverse_binds[j[0]].inverse().transform_point3(Vec3::ZERO));
    let knee = rig.leg_joints.map(|j| rig.inverse_binds[j[1]].inverse().transform_point3(Vec3::ZERO));
    if (thigh[1].x-thigh[0].x).abs()<0.001 { return None; }
    let smooth = |v: f32| { let t = v.clamp(0.0, 1.0); t*t*(3.0-2.0*t) };
    let mut joints = Vec::with_capacity(mesh.positions.len());
    let mut weights = Vec::with_capacity(mesh.positions.len());
    let closed = !coat;
    let closed_leg_share = {
        #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
        if closed && bottom >= 0.70 {
            std::env::var("CRYSTAL_PEOPLE_REVIEW_SKIRT_LEG_SHARE").ok().map(|value| {
                let share: f32=value.parse().expect("skirt leg share must be numeric");
                assert!(share.is_finite() && (0.0..=0.60).contains(&share),
                    "review skirt leg share must be from 0 to 0.60");
                share
            }).unwrap_or(0.60)
        } else {0.60}
        #[cfg(any(not(feature = "people-review"), target_arch = "wasm32"))]
        {0.60}
    };
    for (vertex, p) in mesh.positions.iter().enumerate() {
        let midpoint = (thigh[0].x+thigh[1].x)*0.5;
        let side = smooth(0.5+(p[0]-midpoint)/((thigh[1].x-thigh[0].x)*if coat {0.65} else {1.6}));
        // Closed fabric spans both legs. Keep a pelvis contribution and
        // follow thighs rather than knee hinges so the panels stay continuous.
        let leg = smooth((top-p[1])/0.18) * if closed {closed_leg_share} else {1.0};
        let mut influences = vec![(rig.hips_joint, 1.0-leg)];
        for s in 0..2 {
            let amount = leg * if s==0 { 1.0-side } else { side };
            let shin = if closed {0.0} else {smooth((knee[s].y+0.12-p[1])/0.35)};
            influences.push((rig.leg_joints[s][0], amount*(1.0-shin)));
            influences.push((rig.leg_joints[s][1], amount*shin));
        }
        // The fitted seam interpolates source triangle weights, including
        // Spine as well as Hips. Closed fabric hangs from that waist field;
        // open coats retain their nearest-body blend below the seam.
        // Transfer is performed once at upload, never while the actor moves.
        let transfer = if closed {smooth((p[1]-bottom)/(top-bottom))}
            else {smooth((p[1]-1.20)/0.30)};
        if transfer>0.0 {
            let point = Vec3::from_array(*p);
            let mut nearest: Vec<_> = rig.mesh.positions.iter().enumerate().map(|(i,q)|
                (i,point.distance_squared(Vec3::from_array(*q)))).collect();
            let last = nearest.len().min(8)-1;
            nearest.select_nth_unstable_by(last,|a,b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
            nearest.truncate(8);
            nearest.sort_by(|a,b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
            let total: f32 = nearest.iter().map(|(_,d)| 1.0/(d+0.0025)).sum();
            let mut merged = vec![0.0;rig.skin_nodes.len()];
            for &(j,w) in &influences { merged[j] += w*(1.0-transfer); }
            let drape = if closed && p[1]<top {crate::garment_fit::waist_influences(rig,*p)} else {None};
            if let Some(body) = drape.as_ref().or(seam_weights[vertex].as_ref()) {
                for &(joint, weight) in body { merged[joint] += transfer*weight; }
            } else {
                for &(i,d) in &nearest {
                    let amount = transfer/(d+0.0025)/total;
                    for n in 0..4 { merged[rig.ids[i][n] as usize] += amount*rig.weights[i][n]; }
                }
            }
            influences = merged.into_iter().enumerate().collect();
        }
        influences.sort_by(|a,b| b.1.total_cmp(&a.1));
        let total: f32 = influences[..4].iter().map(|(_,w)| w).sum();
        joints.push(std::array::from_fn(|i| influences[i].0 as u16));
        weights.push(std::array::from_fn(|i| influences[i].1/total));
    }
    if matches!(look.texture_key, "erika" | "falkner" | "kimono_girl") {
        if let Some(obi) = crate::kimono_obi::build(rig, look) {
            let offset=mesh.positions.len() as u32;
            mesh.positions.extend(obi.surface.positions);
            mesh.normals.extend(obi.surface.normals);
            mesh.colors.extend(obi.surface.colors);
            mesh.uvs.extend(obi.surface.uvs);
            mesh.indices.extend(obi.surface.indices.into_iter().map(|i|i+offset));
            joints.extend(obi.joints);weights.extend(obi.weights);
        }
    }
    let mut costume = CostumeMesh { surface: mesh, joints, weights };
    if look.texture_key == "nurse" {
        crate::garment_fit::finish_normals(&mut costume);
    }
    crate::apron::append(&mut costume, look);
    crate::uniform_details::append(&mut costume, rig, look);
    Some(costume)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires supplied male rig in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_elm_fitted_coat_has_finite_outward_surface_normals() {
        use bevy::prelude::*;
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig=crate::open_people::PersonRig::parse(&std::fs::read(root.join("people/kenney/human-male.glb")).unwrap()).unwrap();
        let cloth=costume_mesh(&rig,look(crate::new_bark_actors::character_meshes::CharacterKind::Elm)).unwrap();
        for n in &cloth.surface.normals {
            let n=Vec3::from_array(*n);
            assert!(n.is_finite() && (n.length()-1.).abs()<1e-5);
        }
        let mut edges=std::collections::BTreeMap::new();
        for face in cloth.surface.indices.chunks_exact(3) {
            let p=[face[0],face[1],face[2]].map(|i|Vec3::from_array(cloth.surface.positions[i as usize]));
            let normal=(p[1]-p[0]).cross(p[2]-p[0]);
            assert!(normal.is_finite() && normal.length_squared()>1e-18);
            let shading=face.iter().map(|&i|Vec3::from_array(cloth.surface.normals[i as usize])).sum::<Vec3>();
            assert!(normal.dot(shading)>0., "fitted coat shading faces inward");
            for i in 0..3 {
                let (a,b)=(face[i],face[(i+1)%3]);
                let e=edges.entry((a.min(b),a.max(b))).or_insert((0,0));
                e.0+=1; e.1+=if a<b {1} else {-1};
            }
        }
        assert!(edges.values().all(|&e|e==(2,0)),"coat must remain closed with consistent winding");
    }
    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_tied_hair_is_finite_closed_and_keeps_the_face_open() {
        use bevy::prelude::*;
        use crate::new_bark_actors::character_meshes::SOURCE_KINDS;
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let mut count=0;
        for (_,kind) in SOURCE_KINDS {
            let look=look(kind);
            if !matches!(look.hairstyle,"pigtails"|"rolls"|"ponytail"|"highpony"|"sidepony"|"bun"|"topknot") {continue;}
            count+=1;
            let rig=crate::open_people::PersonRig::parse(&std::fs::read(
                root.join(format!("people/kenney/{}.glb",look.base))).unwrap()).unwrap();
            let mut min=Vec3::splat(f32::INFINITY);let mut max=Vec3::splat(f32::NEG_INFINITY);
            for ((p,joints),weights) in rig.mesh.positions.iter().zip(&rig.ids).zip(&rig.weights) {
                let weight:f32=(0..4).filter(|&i| joints[i] as usize==rig.head_joint).map(|i| weights[i]).sum();
                if weight>0.5 {min=min.min(Vec3::from_array(*p));max=max.max(Vec3::from_array(*p));}
            }
            let center=(min+max)*0.5;let half=(max-min)*0.5;
            let mut mesh=crate::mesh::SurfaceMeshData::default();
            tied_hair(&mut mesh,center,half,look);
            assert!(mesh.positions.len()>1000);
            for (p,n) in mesh.positions.iter().zip(&mesh.normals) {
                let p=Vec3::from_array(*p);let n=Vec3::from_array(*n);
                assert!(p.is_finite() && n.is_finite(),"{} hair surface",look.texture_key);
                assert!((n.length()-1.0).abs()<1e-5);
                assert!(p.z<center.z+half.z*0.4,"{} covers face",look.texture_key);
                assert!(p.distance(center)<half.length()*2.0);
            }
            for face in mesh.indices.chunks_exact(3) {
                let [a,b,c]=[face[0],face[1],face[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let n:Vec3=face.iter().map(|&i| Vec3::from_array(mesh.normals[i as usize])).sum();
                assert!((b-a).cross(c-a).dot(n)>0.0,"{} hair winding",look.texture_key);
            }
            let mut edges=std::collections::HashMap::new();
            for face in mesh.indices.chunks_exact(3) {
                let vertex=|i:u32| mesh.positions[i as usize].map(|p| (p*10000.0).round() as i32);
                for (a,b) in [(face[0],face[1]),(face[1],face[2]),(face[2],face[0])] {
                    let (a,b)=(vertex(a),vertex(b));
                    let edge=if a<b {(a,b)} else {(b,a)};
                    *edges.entry(edge).or_insert(0)+=1;
                }
            }
            assert!(edges.values().all(|&n| n==2),"{} open hair seam",look.texture_key);
            // Each cubic gather begins inside the original head's bounds.
            assert!(mesh.positions.iter().any(|p| {
                let p=Vec3::from_array(*p);p.cmpge(min).all() && p.cmple(max).all()
            }),"{} detached root",look.texture_key);
        }
        assert_eq!(count,16);
    }
    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_glasses_have_valid_wire_surfaces_fitted_to_each_head() {
        use bevy::prelude::*;
        use CharacterKind::*;
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for kind in [Blaine,Elm,Gramps,Granny,Pharmacist,PokefanM,Scientist,SuperNerd] {
            let look=look(kind);
            let rig=crate::open_people::PersonRig::parse(&std::fs::read(
                root.join(format!("people/kenney/{}.glb",look.base))).unwrap()).unwrap();
            let mut min=Vec3::splat(f32::INFINITY);let mut max=Vec3::splat(f32::NEG_INFINITY);
            for ((p,joints),weights) in rig.mesh.positions.iter().zip(&rig.ids).zip(&rig.weights) {
                let weight:f32=(0..4).filter(|&i| joints[i] as usize==rig.head_joint).map(|i| weights[i]).sum();
                if weight>0.5 {min=min.min(Vec3::from_array(*p));max=max.max(Vec3::from_array(*p));}
            }
            let mut mesh=crate::mesh::SurfaceMeshData::default();
            fitted_glasses(&mut mesh,&rig,(min+max)*0.5,(max-min)*0.5,look);
            assert!(mesh.positions.len()>1000);
            for (p,n) in mesh.positions.iter().zip(&mesh.normals) {
                let p=Vec3::from_array(*p);let n=Vec3::from_array(*n);
                assert!(p.is_finite() && n.is_finite());
                assert!((n.length()-1.0).abs()<1e-5);
                assert!(p.cmpge(min-Vec3::splat(0.12)).all() && p.cmple(max+Vec3::splat(0.12)).all());
            }
            for face in mesh.indices.chunks_exact(3) {
                let [a,b,c]=[face[0],face[1],face[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let n=Vec3::from_array(mesh.normals[face[0] as usize]);
                assert!((b-a).cross(c-a).dot(n)>0.0,"{} frame winding",look.texture_key);
            }
        }
    }
    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_jaw_shading_preserves_geometry_and_upper_face_for_the_whole_cast() {
        use bevy::prelude::*;
        use crate::new_bark_actors::character_meshes::SOURCE_KINDS;
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for (_,kind) in SOURCE_KINDS.iter().copied().chain([("outdoorsman",CharacterKind::Outdoorsman)]) {
            let look=look(kind);
            let rig=crate::open_people::PersonRig::parse(&std::fs::read(
                root.join(format!("people/kenney/{}.glb",look.base))).unwrap()).unwrap();
            let mut mesh=rig.mesh.clone();let mut ids=rig.ids.clone();let mut weights=rig.weights.clone();
            child_head(&rig,&mut mesh,&mut ids,&mut weights,look);
            let original=mesh.clone();let original_ids=ids.clone();let original_weights=weights.clone();
            soften_jaw_normals(&rig,&mut mesh,&ids,&weights);
            assert_eq!(mesh.positions,original.positions);assert_eq!(mesh.indices,original.indices);
            assert_eq!(mesh.uvs,original.uvs);assert_eq!(mesh.colors,original.colors);
            assert_eq!(ids,original_ids);assert_eq!(weights,original_weights);
            let mut improved=0;
            for i in 0..mesh.positions.len() {
                let n=Vec3::from_array(mesh.normals[i]);let old=Vec3::from_array(original.normals[i]);
                assert!(n.is_finite() && (n.length()-1.0).abs()<0.001);
                let head=(0..4).filter(|&j|ids[i][j] as usize==rig.head_joint)
                    .map(|j|weights[i][j]).sum::<f32>()>0.5;
                if !head || old.y>=-0.30 || mesh.positions[i][1]>3.16 {
                    assert_eq!(n,old,"{} body or upper face shading changed",look.texture_key);
                }
                if n!=old {
                    assert!(head && n.y>old.y && n.y<0.0);
                    assert!(n.dot(old)>0.4,"{} flipped lower-face shading",look.texture_key);
                    improved+=1;
                }
            }
            assert!(improved>10,"{} lower face unchanged",look.texture_key);
        }
    }
    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_rear_fabric_preserves_front_and_skin_for_the_whole_cast() {
        use crate::new_bark_actors::character_meshes::SOURCE_KINDS;
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let mut changed=0;
        for (_,kind) in SOURCE_KINDS {
            let look=look(kind);
            let rig=crate::open_people::PersonRig::parse(&std::fs::read(
                root.join(format!("people/kenney/{}.glb",look.base))).unwrap()).unwrap();
            let mut body=rig.mesh.clone();let mut joints=rig.ids.clone();let mut weights=rig.weights.clone();
            correct_body_uvs(&mut body,look,&mut joints,&mut weights);
            let intended=matches!(look.texture_key,"chris"|"lass"|"cooltrainer_f"|"nurse"|"gramps"|"super_nerd");
            if intended { assert!(body.positions.len()>rig.mesh.positions.len(),"{} missing rear correction",look.texture_key);changed+=1; }
            else { assert_eq!(body.positions.len(),rig.mesh.positions.len());assert_eq!(body.indices,rig.mesh.indices);assert_eq!(body.uvs,rig.mesh.uvs); }
            assert_eq!(body.positions.len(),joints.len());assert_eq!(joints.len(),weights.len());
            for (original,updated) in rig.mesh.indices.chunks_exact(3).zip(body.indices.chunks_exact(3)) {
                let rear=original.iter().map(|&i|rig.mesh.positions[i as usize][2]).sum::<f32>()/3.0;
                for (&old,&new) in original.iter().zip(updated) {
                    let (old,new)=(old as usize,new as usize);
                    assert_eq!(body.positions[new],rig.mesh.positions[old]);
                    assert_eq!(body.normals[new],rig.mesh.normals[old]);
                    assert_eq!(body.colors[new],rig.mesh.colors[old]);
                    assert_eq!(joints[new],rig.ids[old]);assert_eq!(weights[new],rig.weights[old]);
                    if new!=old {
                        assert!(intended && rear<0.03);
                        assert!(body.uvs[new].iter().all(|v|v.is_finite() && (0.0..=1.0).contains(v)));
                        assert!(rig.mesh.uvs[old][0]<0.625 && rig.mesh.uvs[old][1]>0.47);
                    } else { assert_eq!(body.uvs[new],rig.mesh.uvs[old]); }
                    if rear>=0.03 || rig.mesh.positions[old][2]>0.1
                        || rig.mesh.positions[old][0].abs()>=0.4
                        || (look.texture_key!="chris" && rig.mesh.uvs[old][0]>=0.5) {
                        assert_eq!(body.uvs[new],rig.mesh.uvs[old],"{} front changed",look.texture_key);
                    }
                }
            }
        }
        assert_eq!(changed,6);
    }
    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_backpacks_preserve_body_skin_and_attachment_bind_space() {
        use bevy::prelude::*;
        use CharacterKind::*;
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for kind in [Trainer,TrainerFemale,Cal,Red,Outdoorsman,Brock] {
            let look=look(kind);
            let rig=crate::open_people::PersonRig::parse(&std::fs::read(
                root.join(format!("people/kenney/{}.glb",look.base))).unwrap()).unwrap();
            let mut body=rig.mesh.clone();let mut joints=rig.ids.clone();let mut weights=rig.weights.clone();
            correct_body_uvs(&mut body,look,&mut joints,&mut weights);
            assert_eq!(body.positions.len(),joints.len());assert_eq!(body.positions.len(),weights.len());
            for (&old,&new) in rig.mesh.indices.iter().zip(&body.indices) {
                let (old,new)=(old as usize,new as usize);
                assert_eq!(body.positions[new],rig.mesh.positions[old]);
                assert_eq!(body.normals[new],rig.mesh.normals[old]);
                assert_eq!(joints[new],rig.ids[old]);assert_eq!(weights[new],rig.weights[old]);
                if rig.mesh.positions[old][2]>0.1 || look.texture_key!="chris" {
                    assert_eq!(body.uvs[new],rig.mesh.uvs[old],"front or unrelated UV changed");
                }
            }
            let bag=backpack_mesh(&rig,look).unwrap();
            assert!(bag.positions.len()>1000);
            assert_eq!(bag.positions.len(),bag.normals.len());
            let inverse=rig.inverse_binds[rig.chest_joint].inverse();
            for (p,n) in bag.positions.iter().zip(&bag.normals) {
                let point=inverse.transform_point3(Vec3::from_array(*p));
                assert!(point.is_finite() && (1.4..2.6).contains(&point.y));
                assert!(point.x.abs()<0.4 && (-0.6..0.3).contains(&point.z));
                let normal=Vec3::from_array(*n);
                assert!(normal.is_finite() && (normal.length()-1.0).abs()<1e-5);
            }
        }
    }
    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_costumes_keep_bind_shape_and_follow_the_animated_legs() {
        use bevy::prelude::*;
        use crate::new_bark_actors::character_meshes::SOURCE_KINDS;
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let load = |name| crate::open_people::PersonRig::parse(
            &std::fs::read(root.join(format!("people/kenney/{name}.glb"))).unwrap()).unwrap();
        let male = load("human-male");
        let female = load("human-female");
        fn world(i: usize, pose: &[Transform], parents: &[Option<usize>]) -> Mat4 {
            parents[i].map(|p| world(p,pose,parents)).unwrap_or(Mat4::IDENTITY)*pose[i].to_matrix()
        }
        let mut checked = 0;
        for (_,kind) in SOURCE_KINDS {
            let look = look(kind);
            let rig = if look.base=="human-female" { &female } else { &male };
            if !matches!(look.costume,"dress"|"kimono"|"robe"|"coat") && !look.accessories.contains("skirt") {continue;}
            let Some(cloth) = costume_mesh(rig,look) else { continue; };
            checked += 1;
            assert_eq!(cloth.surface.positions.len(),cloth.joints.len());
            assert_eq!(cloth.joints.len(),cloth.weights.len());
            let mut pose = rig.nodes.clone();
            rig.sample("Run",rig.duration("Run")*0.25,&mut pose);
            let mesh_inverse = world(rig.mesh_node,&pose,&rig.parents).inverse();
            let skin: Vec<_> = rig.skin_nodes.iter().zip(&rig.inverse_binds)
                .map(|(&node,bind)| mesh_inverse*world(node,&pose,&rig.parents)* *bind).collect();
            let rest_inverse = world(rig.mesh_node,&rig.nodes,&rig.parents).inverse();
            let neutral: Vec<_> = rig.skin_nodes.iter().zip(&rig.inverse_binds)
                .map(|(&node,bind)| rest_inverse*world(node,&rig.nodes,&rig.parents)* *bind).collect();
            let mut moving_hem = false;
            let hips = skin[rig.hips_joint];
            for ((p,ids),weights) in cloth.surface.positions.iter().zip(&cloth.joints).zip(&cloth.weights) {
                assert!(weights.iter().all(|w| w.is_finite() && (0.0..=1.0).contains(w)));
                assert!((weights.iter().sum::<f32>()-1.0).abs()<1e-5);
                assert!(ids.iter().all(|&j| (j as usize)<rig.skin_nodes.len()));
                let p = Vec3::from_array(*p);
                let deform = |matrices: &[Mat4]| (0..4).map(|i|
                    matrices[ids[i] as usize].transform_point3(p)*weights[i]).sum::<Vec3>();
                assert!(deform(&neutral).distance(p)<0.002,"{} changes bind shape",look.texture_key);
                let animated = deform(&skin);
                assert!(animated.is_finite());
                if p.y>1.63 {
                    assert!(ids.iter().zip(weights).any(|(&id,&weight)| id as usize!=rig.hips_joint && weight>0.1),
                        "{} seam must share torso influences",look.texture_key);
                }
                moving_hem |= p.y<1.0 && animated.distance(hips.transform_point3(p))>0.05;
            }
            assert!(moving_hem,"{} still has a rigid garment",look.texture_key);
            // Exercise both planted and raised legs over the complete cycle;
            // costume culling must cover its deformed surface at every sample.
            for step in 0..8 {
                rig.sample("Run",rig.duration("Run")*step as f32/8.0,&mut pose);
                let mesh_inverse = world(rig.mesh_node,&pose,&rig.parents).inverse();
                let skin: Vec<_> = rig.skin_nodes.iter().zip(&rig.inverse_binds)
                    .map(|(&node,bind)| mesh_inverse*world(node,&pose,&rig.parents)* *bind).collect();
                for ((p,ids),weights) in cloth.surface.positions.iter().zip(&cloth.joints).zip(&cloth.weights) {
                    let p = Vec3::from_array(*p);
                    let animated = (0..4).map(|i| skin[ids[i] as usize].transform_point3(p)*weights[i]).sum::<Vec3>();
                    assert!(animated.is_finite());
                    assert!(animated.cmpge(rig.bounds.0-Vec3::splat(0.6)).all()
                        && animated.cmple(rig.bounds.1+Vec3::splat(0.6)).all(),
                        "{} garment escapes its render bounds",look.texture_key);
                }
            }
        }
        assert_eq!(checked,29,"complete dress/coat roster was not exercised");
    }
    #[test]
    fn tailored_profiles_have_finite_smooth_normals_and_consistent_winding() {
        use bevy::prelude::*;
        let mut mesh = crate::mesh::SurfaceMeshData::default();
        let profiles = [(1.6, 0.36, 0.32), (1.2, 0.43, 0.35), (0.8, 0.52, 0.37)];
        profile_shell(&mut mesh, 0.0, 0.05, &profiles,
            (0.0, std::f32::consts::TAU), 0.018, [90, 100, 110]);
        for normal in &mesh.normals {
            let n = Vec3::from_array(*normal);
            assert!(n.is_finite());
            assert!((n.length()-1.0).abs()<1e-5);
        }
        // Outer and inner surfaces must both agree with their lit normals.
        // Rims use interpolated garment normals for the soft rolled edge.
        for ids in mesh.indices[..(profiles.len()-1)*40*12].chunks_exact(3) {
            let p = [ids[0], ids[1], ids[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
            let n = Vec3::from_array(mesh.normals[ids[0] as usize]);
            assert!((p[1]-p[0]).cross(p[2]-p[0]).dot(n)>0.0);
        }
        let mut open = crate::mesh::SurfaceMeshData::default();
        profile_shell(&mut open, 0., 0.05, &profiles,
            (std::f32::consts::PI*0.13,std::f32::consts::PI*1.87),0.018,[90,100,110]);
        for shell in [mesh,open] {
            // Weld the full-circle atlas seam before checking topology.
            let keys: Vec<[i32;3]>=shell.positions.iter().map(|p|p.map(|v|(v*100000.).round() as i32)).collect();
            let mut edges=std::collections::BTreeMap::new();
            for face in shell.indices.chunks_exact(3) {
                let p=[face[0],face[1],face[2]].map(|i|Vec3::from_array(shell.positions[i as usize]));
                assert!((p[1]-p[0]).cross(p[2]-p[0]).length_squared()>1e-18);
                for i in 0..3 {
                    let (a,b)=(keys[face[i] as usize],keys[face[(i+1)%3] as usize]);
                    let e=edges.entry((a.min(b),a.max(b))).or_insert((0,0));
                    e.0+=1;e.1+=if a<b {1}else{-1};
                }
            }
            assert!(edges.values().all(|&e|e==(2,0)),"garment rim winding must close both surfaces");
        }
    }
    #[test]
    fn hair_curtains_are_connected_behind_the_head_and_leave_the_face_open() {
        use bevy::prelude::*;
        for long in [false, true] {
            let mut mesh = crate::mesh::SurfaceMeshData::default();
            hair_curtain(&mut mesh, Vec3::new(0.0, 3.18, 0.0),
                Vec3::new(0.44, 0.58, 0.52), long, [76, 43, 31], None);
            assert!(mesh.positions.iter().any(|p| p[0].abs()<0.01 && p[2] < -0.5));
            assert!(mesh.positions.iter().all(|p| p[2]<0.29));
            assert!(mesh.positions.iter().all(|p| Vec3::from_array(*p).is_finite()));
            assert!(mesh.normals.iter().all(|n| {
                let n=Vec3::from_array(*n);n.is_finite() && (n.length()-1.0).abs()<1e-4
            }));
            assert!(mesh.indices.iter().all(|&i| (i as usize)<mesh.positions.len()));
            // Main inner/outer surfaces must agree with their lighting normals.
            for ids in mesh.indices[..56*64*12].chunks_exact(3) {
                let p=[ids[0],ids[1],ids[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let n=Vec3::from_array(mesh.normals[ids[0] as usize]);
                assert!((p[1]-p[0]).cross(p[2]-p[0]).dot(n)>0.0);
            }
            let mut edges=std::collections::HashMap::new();
            for ids in mesh.indices.chunks_exact(3) {
                for (a,b) in [(ids[0],ids[1]),(ids[1],ids[2]),(ids[2],ids[0])] {
                    *edges.entry((a.min(b),a.max(b))).or_insert(0)+=1;
                }
            }
            assert!(edges.values().all(|&count| count==2),"open curtain seam");
        }
    }
    #[test]
    fn painted_curtains_sample_only_hair_and_preserve_closed_geometry() {
        use bevy::prelude::*;
        // Bright green represents forbidden skin/body/neighboring islands.
        let mut rgba=[0,255,0,255].repeat(256*256);
        for y in 0..52 {for x in 0..36 {
            let at=(y*256+x)*4;
            rgba[at..at+4].copy_from_slice(&[80+x as u8,40+y as u8/4,30,255]);
        }}
        let painting=crate::npc_textures::Texture {size:(256,256),rgba};
        for long in [false,true] {
            let mut plain=crate::mesh::SurfaceMeshData::default();
            let mut painted=crate::mesh::SurfaceMeshData::default();
            let center=Vec3::new(0.0,3.18,0.0);
            let half=Vec3::new(0.44,0.58,0.52);
            hair_curtain(&mut plain,center,half,long,[76,43,31],None);
            hair_curtain(&mut painted,center,half,long,[76,43,31],Some(&painting));
            assert_eq!(painted.positions,plain.positions);
            assert_eq!(painted.normals,plain.normals);
            assert_eq!(painted.indices,plain.indices);
            assert_eq!(painted.uvs,plain.uvs);
            assert!(painted.colors.iter().all(|c| c[0]>c[1] && c[1]>c[2]
                && c.iter().all(|v| v.is_finite()) && c[3]==1.0));
            assert!(painted.colors.iter().any(|c| *c!=painted.colors[0]),
                "painted hair variation lost");
        }
    }
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
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_female_fringe_recolors_without_changing_eyes_brows_or_mouth() {
        use CharacterKind::*;
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig=crate::open_people::PersonRig::parse(&std::fs::read(
            root.join("people/kenney/human-female.glb")).unwrap()).unwrap();
        for kind in [TrainerFemale,Misty,Janine,Whitney,Beauty,Clair,Daisy,Erika,Jasmine,Karen,
            KimonoGirl,LinkReceptionist,Mom,OldLinkReceptionist,PokefanF,Receptionist,RedsMom,
            RocketGirl,Sabrina,SwimmerGirl,Teacher,Twin] {
            let result=texture(&rig.rgba,rig.texture_size,look(kind));
            let mut changed=0;
            for i in 0..result.len()/4 {
                let x=(i as u32%rig.texture_size.0)*256/rig.texture_size.0;
                let y=(i as u32/rig.texture_size.0)*256/rig.texture_size.1;
                let p=&rig.rgba[i*4..i*4+4];let q=&result[i*4..i*4+4];
                if ((60..100).contains(&x) && (48..56).contains(&y))
                    || ((48..=118).contains(&x) && (56..125).contains(&y)) {
                    assert_eq!(q,p,"{} facial feature changed at {x},{y}",look(kind).texture_key);
                }
                if (48..=118).contains(&x) && (30..56).contains(&y)
                    && (y<48 || x<60 || x>=100) && p[0]<125 && p[1]<120 && p[2]<115 {
                    changed+=usize::from(p!=q);
                    assert_ne!(p,q,"{} hair unchanged at {x},{y}",look(kind).texture_key);
                }
                assert_eq!(p[3],q[3]);
            }
            assert!(changed>100,"{} fringe unchanged",look(kind).texture_key);
        }
    }
    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_male_hair_recolors_forelock_and_sideburns_and_preserves_features() {
        use CharacterKind::*;
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let male=crate::open_people::PersonRig::parse(&std::fs::read(
            root.join("people/kenney/human-male.glb")).unwrap()).unwrap();
        let kinds=crate::new_bark_actors::character_meshes::SOURCE_KINDS.iter()
            .map(|&(_,kind)|kind).chain([Outdoorsman]);
        for kind in kinds {
            let look=look(kind);
            if look.base!="human-male" || matches!(look.hairstyle,"bald"|"sparse")
                || matches!(kind,Trainer|SuperNerd|Youngster) {continue;}
            let result=texture(&male.rgba,male.texture_size,look);
            let mut changed=0;
            for i in 0..result.len()/4 {
                let x=(i as u32%male.texture_size.0)*256/male.texture_size.0;
                let y=(i as u32/male.texture_size.0)*256/male.texture_size.1;
                let p=&male.rgba[i*4..i*4+4];let q=&result[i*4..i*4+4];
                if ((61..96).contains(&x) && (43..125).contains(&y))
                    || ((106..115).contains(&x) && (56..76).contains(&y))
                    || ((48..51).contains(&x) && (56..76).contains(&y)) {
                    assert_eq!(p,q,"{} facial/ear pixel changed at {x},{y}",look.texture_key);
                }
                if (48..119).contains(&x) && (30..43).contains(&y)
                    && p[0]<125 && p[1]<120 && p[2]<115 {
                    assert_ne!(p,q,"{} forelock unchanged at {x},{y}",look.texture_key);
                    changed+=1;
                }
                assert_eq!(p[3],q[3]);
            }
            assert!(changed>400,"{} forelock missing",look.texture_key);
            for (x,y) in [(53,58),(102,58),(98,47)] {
                let i=((y*male.texture_size.1/256)*male.texture_size.0
                    + x*male.texture_size.0/256) as usize*4;
                assert_ne!(&result[i..i+3],&male.rgba[i..i+3],
                    "{} temple/sideburn unchanged at {x},{y}",look.texture_key);
            }
        }
    }
    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_child_faces_clean_chins_and_keep_features_and_adult_textures() {
        use CharacterKind::*;
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let load=|name:&str| crate::open_people::PersonRig::parse(&std::fs::read(
            root.join(format!("people/kenney/{name}.glb"))).unwrap()).unwrap();
        let male=load("human-male");let clean=load("human-female");
        for kind in [Bugsy,BugCatcher,GameboyKid,StandingYoungster,Biker,Elm,Trainer,Youngster] {
            let look=look(kind);let original=texture(&male.rgba,male.texture_size,look);
            let mut result=original.clone();
            clean_child_face(&mut result,male.texture_size,look,&clean.rgba,clean.texture_size);
            if matches!(kind,Biker|Elm|Trainer|Youngster) {assert_eq!(result,original);continue;}
            let mut changed=0;
            for i in 0..result.len()/4 {
                let x=(i as u32%male.texture_size.0)*256/male.texture_size.0;
                let y=(i as u32/male.texture_size.0)*256/male.texture_size.1;
                let p=&original[i*4..i*4+4];let q=&result[i*4..i*4+4];
                if !(51..108).contains(&x) || !(63..99).contains(&y)
                    || ((74..84).contains(&x) && y<65)
                    || ((70..88).contains(&x) && (65..71).contains(&y)
                        && f32::from(p[0])>f32::from(p[1])*1.35
                        && f32::from(p[0])>f32::from(p[2])*1.2) {
                    assert_eq!(p,q,"{} feature or clothing changed at {x},{y}",look.texture_key);
                }
                changed+=usize::from(p!=q);assert_eq!(p[3],q[3]);
            }
            assert!(changed>1000);
            // Fully replaced chin samples must equal the existing clean atlas.
            for (x,y) in [(65,79),(80,84),(95,79)] {
                let at=((y*256+x)*4) as usize;
                assert_eq!(&result[at..at+3],&clean.rgba[at..at+3]);
                assert_ne!(&result[at..at+3],&original[at..at+3]);
            }
        }
    }
    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_child_face_style_preserves_hair_ears_clothing_and_adults() {
        use CharacterKind::*;
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let load=|name:&str|crate::open_people::PersonRig::parse(&std::fs::read(
            root.join(format!("people/kenney/{name}.glb"))).unwrap()).unwrap();
        let rig=load("human-male");let clean=load("human-female");
        let painted=crate::npc_textures::decode(&std::fs::read(root.join("people/npc-textures/youngster.png")).unwrap()).unwrap();
        for kind in [Bugsy,BugCatcher,GameboyKid,StandingYoungster,Elm,Biker,Trainer,Youngster] {
            let look=look(kind);let mut original=texture(&rig.rgba,rig.texture_size,look);
            clean_child_face(&mut original,rig.texture_size,look,&clean.rgba,clean.texture_size);
            let mut result=original.clone();paint_child_face(&mut result,rig.texture_size,look,&painted.rgba,painted.size);
            if matches!(kind,Elm|Biker|Trainer|Youngster) {assert_eq!(result,original);continue;}
            let at=|x:u32,y:u32|((y*rig.texture_size.1/256)*rig.texture_size.0+x*rig.texture_size.0/256) as usize*4;
            let pixel=|x,y|&result[at(x,y)..at(x,y)+3];
            assert!(pixel(68,56).iter().map(|&v|u32::from(v)).sum::<u32>()
                <pixel(80,55).iter().map(|&v|u32::from(v)).sum::<u32>()/2,"{} eye contrast",look.texture_key);
            let mouth=pixel(78,69);assert!(f32::from(mouth[0])>f32::from(mouth[1])*1.3);
            assert_ne!(pixel(68,56),&original[at(68,56)..at(68,56)+3]);
            assert_ne!(pixel(78,69),&original[at(78,69)..at(78,69)+3]);
            for (x,y) in [(80,40),(55,47),(104,47),(49,64),(111,64),(20,110),(80,160),(210,220)] {
                assert_eq!(&result[at(x,y)..at(x,y)+4],&original[at(x,y)..at(x,y)+4],
                    "{} hair, ear or clothing changed at {x},{y}",look.texture_key);
            }
            assert!(result.chunks_exact(4).zip(original.chunks_exact(4)).filter(|(a,b)|a!=b).count()>1000);
            assert!(result.chunks_exact(4).zip(original.chunks_exact(4)).all(|(a,b)|a[3]==b[3]));
        }
    }
    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_shorts_have_skin_legs_shared_hems_and_unchanged_shoes() {
        use CharacterKind::*;
        use bevy::prelude::*;
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for kind in [BugCatcher,Bugsy,GameboyKid,StandingYoungster,Youngster,Misty,Whitney,SwimmerGuy,Elm,Trainer] {
            let look=look(kind);
            let rig=crate::open_people::PersonRig::parse(&std::fs::read(
                root.join(format!("people/kenney/{}.glb",look.base))).unwrap()).unwrap();
            let mut mesh=rig.mesh.clone();let mut ids=rig.ids.clone();let mut weights=rig.weights.clone();
            shorts_body(&rig,&mut mesh,&mut ids,&mut weights,look);
            if matches!(kind,Elm|Trainer) {
                assert_eq!(mesh.positions,rig.mesh.positions);assert_eq!(mesh.indices,rig.mesh.indices);
                assert_eq!(mesh.uvs,rig.mesh.uvs);continue;
            }
            let at=|j:usize|rig.inverse_binds[j].inverse().transform_point3(Vec3::ZERO).y;
            let knee=at(rig.leg_joints[0][1]);let hem=knee+(at(rig.leg_joints[0][0])-knee)*0.47;
            assert!(mesh.positions.len()>rig.mesh.positions.len());
            assert_eq!(mesh.positions.len(),ids.len());assert_eq!(ids.len(),weights.len());
            for i in 0..rig.mesh.positions.len() {
                assert_eq!(mesh.positions[i],rig.mesh.positions[i]);
                if rig.mesh.uvs[i][0]<0.605 || rig.mesh.uvs[i][1]<0.745 {
                    assert_eq!(mesh.normals[i],rig.mesh.normals[i]);
                }
                assert_eq!(mesh.uvs[i],rig.mesh.uvs[i]);assert_eq!(ids[i],rig.ids[i]);assert_eq!(weights[i],rig.weights[i]);
            }
            let mut bare=0;let mut hem_pairs=0;let mut rim=0;
            for i in rig.mesh.positions.len()..mesh.positions.len() {
                assert!(Vec3::from_array(mesh.positions[i]).is_finite());
                assert!((Vec3::from_array(mesh.normals[i]).length()-1.0).abs()<0.001);
                assert!((weights[i].iter().sum::<f32>()-1.0).abs()<0.001);
                assert!(ids[i].iter().all(|&j|(j as usize)<rig.inverse_binds.len()));
                if mesh.normals[i]==[0.0,-1.0,0.0] && (mesh.positions[i][1]-hem).abs()<1e-5
                    && mesh.uvs[i][1]>=0.745 {rim+=1;}
                if (0.89..0.96).contains(&mesh.uvs[i][0]) && mesh.uvs[i][1]<0.5 {
                    bare+=1;assert!(mesh.positions[i][1]<=hem+1e-5);
                    if (mesh.positions[i][1]-hem).abs()<1e-5 {
                        assert!((0..i).any(|j|mesh.positions[j]==mesh.positions[i]
                            && ids[j]==ids[i] && weights[j]==weights[i] && mesh.uvs[j][1]>=0.745));
                        hem_pairs+=1;
                    }
                }
            }
            assert!(bare>100 && hem_pairs>10 && rim>40,"{} missing shorts/hem",look.texture_key);
            let mut hem_edges=std::collections::HashMap::new();
            let key=|i:usize|mesh.positions[i].map(|v|(v*10_000.0).round() as i32);
            for face in mesh.indices.chunks_exact(3) {for edge in 0..3 {
                let a=face[edge] as usize;let b=face[(edge+1)%3] as usize;
                if (mesh.positions[a][1]-hem).abs()>1e-5 || (mesh.positions[b][1]-hem).abs()>1e-5 {continue;}
                let a=key(a);let b=key(b);if a==b {continue;}
                *hem_edges.entry((a.min(b),a.max(b))).or_insert(0usize)+=1;
            }}
            assert!(hem_edges.values().all(|&count|count==2),"{} open fabric rim {:?}",look.texture_key,
                hem_edges.iter().filter(|(_,count)|**count!=2).take(12).collect::<Vec<_>>());
            for face in rig.mesh.indices.chunks_exact(3) {
                if face.iter().all(|&i|rig.mesh.positions[i as usize][1]<0.24) {
                    assert!(mesh.indices.chunks_exact(3).any(|f|f==face),"{} shoe changed",look.texture_key);
                }
            }
            for face in mesh.indices.chunks_exact(3) {
                let p=[face[0],face[1],face[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                let n=face.iter().map(|&i|Vec3::from_array(mesh.normals[i as usize])).sum::<Vec3>();
                assert!((p[1]-p[0]).cross(p[2]-p[0]).dot(n)>=-1e-6,"{} winding",look.texture_key);
            }
        }
    }
    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_child_jaws_keep_body_skinning_and_have_finite_smooth_surfaces() {
        use CharacterKind::*;
        use bevy::prelude::*;
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig=crate::open_people::PersonRig::parse(&std::fs::read(
            root.join("people/kenney/human-male.glb")).unwrap()).unwrap();
        let is_head=|i:usize| (0..4).filter(|&j|rig.ids[i][j] as usize==rig.head_joint)
            .map(|j|rig.weights[i][j]).sum::<f32>()>0.5;
        let original_min=rig.mesh.positions.iter().enumerate().filter(|(i,_)|is_head(*i))
            .map(|(_,p)|p[1]).fold(f32::INFINITY,f32::min);
        for kind in [Bugsy,BugCatcher,GameboyKid,StandingYoungster,Elm,Trainer] {
            let mut mesh=rig.mesh.clone();let mut ids=rig.ids.clone();let mut weights=rig.weights.clone();
            child_head(&rig,&mut mesh,&mut ids,&mut weights,look(kind));
            if matches!(kind,Elm|Trainer) {
                assert_eq!(mesh.positions,rig.mesh.positions);assert_eq!(mesh.indices,rig.mesh.indices);
                assert_eq!(mesh.normals,rig.mesh.normals);continue;
            }
            assert!(mesh.positions.len()>rig.mesh.positions.len());
            assert_eq!(mesh.positions.len(),ids.len());assert_eq!(ids.len(),weights.len());
            for i in 0..rig.mesh.positions.len() {
                assert_eq!(ids[i],rig.ids[i]);assert_eq!(weights[i],rig.weights[i]);
                assert_eq!(mesh.uvs[i],rig.mesh.uvs[i]);
                if !is_head(i) {
                    assert_eq!(mesh.positions[i],rig.mesh.positions[i]);
                    assert_eq!(mesh.normals[i],rig.mesh.normals[i]);
                }
            }
            for (p,n) in mesh.positions.iter().zip(&mesh.normals) {
                assert!(Vec3::from_array(*p).is_finite());let n=Vec3::from_array(*n);
                assert!(n.is_finite() && (n.length()-1.0).abs()<1e-4);
            }
            for w in &weights {assert!(w.iter().all(|v|v.is_finite() && *v>=0.0));
                assert!((w.iter().sum::<f32>()-1.0).abs()<1e-4);}
            let min=mesh.positions.iter().enumerate().filter(|(i,_)|
                (0..4).filter(|&j|ids[*i][j] as usize==rig.head_joint)
                    .map(|j|weights[*i][j]).sum::<f32>()>0.5)
                .map(|(_,p)|p[1]).fold(f32::INFINITY,f32::min);
            assert!(min>original_min+0.02,"jaw did not shorten");
            for face in mesh.indices.chunks_exact(3) {
                let p=[face[0],face[1],face[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                let geometric=(p[1]-p[0]).cross(p[2]-p[0]);
                if geometric.length_squared()>1e-12 {
                    let n=face.iter().map(|&i|Vec3::from_array(mesh.normals[i as usize])).sum::<Vec3>();
                    assert!(geometric.dot(n)>-1e-6,"reversed child face");
                }
            }
        }
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
