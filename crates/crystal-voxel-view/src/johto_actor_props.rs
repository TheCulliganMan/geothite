//! Original source-scoped volumetric actor assets, with no simulation state.
//!
//! Runtime source identity is published after variable-sprite and icon
//! resolution. `monster` and `icon_monster` therefore remain different keys.
//! Models are ground rooted (+Y up, front +Z) and never infer species from
//! colors, original script tokens, actor IDs, or another actor's appearance.
use std::sync::OnceLock;

use crate::mesh::SurfaceMeshData;
use bevy::prelude::Vec3;
use serde::Deserialize;

/// Native prop sprites already contain their current map/time palette.
/// Ignore transparent background and black outlines, retaining a readable
/// body tone from the live palette rather than guessing from the map name.
pub(crate) fn native_stone_color(image: &bevy::prelude::Image) -> Option<[u8;3]> {
    use bevy::render::render_resource::TextureFormat;
    if !matches!(image.texture_descriptor.format,
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb) {return None;}
    let data=image.data.as_ref()?;
    if data.len()!=image.width() as usize*image.height() as usize*4 {return None;}
    let mut counts=std::collections::BTreeMap::<[u8;3],usize>::new();
    for p in data.chunks_exact(4) {
        if p[3]!=255 || u32::from(p[0])+u32::from(p[1])+u32::from(p[2])<96 {continue;}
        *counts.entry([p[0],p[1],p[2]]).or_default()+=1;
    }
    let colored=counts.iter().any(|(rgb,count)|*count>=4
        && rgb.iter().max().unwrap()-rgb.iter().min().unwrap()>=16);
    let tones=counts.into_iter().filter(|(rgb,count)|*count>=4
        && if colored {rgb.iter().max().unwrap()-rgb.iter().min().unwrap()>=16}
            else {*rgb.iter().max().unwrap()>=96}).collect::<Vec<_>>();
    let &(dominant,_)=tones.iter().max_by_key(|(_,count)|*count)?;
    // Dark live palettes can have a nearly black body and a readable mineral
    // highlight. Baking face shade over only that dark body erases the volume.
    // Use an actual significant highlight, never invent a night exposure/hue.
    if *dominant.iter().max().unwrap()<96 {
        tones.into_iter().max_by_key(|(rgb,count)|
            (u32::from(rgb[0])*2126+u32::from(rgb[1])*7152+u32::from(rgb[2])*722,*count))
            .map(|(rgb,_)|rgb)
    } else {Some(dominant)}
}

pub(crate) fn native_stone_mesh(kind: PropKind, rgb: [u8;3]) -> SurfaceMeshData {
    let mut data=mesh(kind);
    let mut low=Vec3::splat(f32::INFINITY);let mut high=Vec3::splat(f32::NEG_INFINITY);
    for p in &data.positions {let p=Vec3::from_array(*p);low=low.min(p);high=high.max(p);}
    let paint=rgb.map(|v| {let v=f32::from(v)/255.;
        if v<=0.04045 {v/12.92}else{((v+0.055)/1.055).powf(2.4)}});
    // Loose smashable stone keeps the live hue, with a little neutral mineral
    // in its coating so ochre source sprites read as rock rather than gold.
    let paint=if kind==PropKind::Rock {
        let value=paint[0]*0.2126+paint[1]*0.7152+paint[2]*0.0722;
        paint.map(|c|c*0.72+value*0.28)
    } else {paint};
    for i in 0..data.colors.len() {
        let old=data.colors[i];
        let pigment=((old[0]*0.2126+old[1]*0.7152+old[2]*0.0722)/0.30).clamp(0.35,1.10);
        let facing=0.66+0.34*Vec3::from_array(data.normals[i])
            .dot(Vec3::new(-0.35,0.85,0.40).normalize()).max(0.);
        data.colors[i]=[paint[0]*pigment*facing,paint[1]*pigment*facing,paint[2]*pigment*facing,old[3]];
        let p=(Vec3::from_array(data.positions[i])-low)/(high-low).max(Vec3::splat(0.0001));
        let n=Vec3::from_array(data.normals[i]).abs();
        let uv=if n.y>=n.x && n.y>=n.z {[p.x,p.z]}
            else if n.x>=n.z {[p.z,p.y]} else {[p.x,p.y]};
        data.uvs[i]=crate::scenery_finish::source_atlas_uv(uv);
    }
    data
}

#[derive(Deserialize)]
struct Primitive {
    #[serde(default)]
    material: String,
    positions: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
    base_color: [f32; 4],
}
#[derive(Deserialize)]
struct Export {
    #[serde(default)]
    name: String,
    primitives: Vec<Primitive>,
}

/// Parse our original authoring format without discarding smooth normals.
/// Kept render-only so battle and world presentation can share the same art.
fn parse_model<'a>(
    json: impl Into<crate::model_storage::Source<'a>>,
) -> Result<SurfaceMeshData, String> {
    parse_model_palette(json, false)
}

fn parse_model_palette<'a>(
    json: impl Into<crate::model_storage::Source<'a>>,
    gold_gym_stone: bool,
) -> Result<SurfaceMeshData, String> {
    let source: Export = crate::model_storage::parse(json)?;
    if gold_gym_stone && source.name != "boulder" {
        return Err("gym palette requires the named boulder asset".into());
    }
    let mut mesh = SurfaceMeshData::default();
    for p in source.primitives {
        let color = if gold_gym_stone {
            let rgb = match p.material.as_str() {
                "stone" => [165, 123, 24],
                "stone_light" => [198, 148, 57],
                "stone_dark" => [57, 57, 57],
                _ => return Err("gym boulder requires named stone materials".into()),
            };
            // Verified Cianwood/Blackthorn sprite shades; PBR vertex colors are linear.
            let linear = |v: u8| {
                let v = f32::from(v) / 255.;
                if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
            };
            [linear(rgb[0]), linear(rgb[1]), linear(rgb[2]), 1.]
        } else { p.base_color };
        let n = p.positions.len() / 3;
        if n < 3
            || p.positions.len() % 3 != 0
            || p.normals.len() != p.positions.len()
            || p.indices.is_empty()
            || p.indices.len() % 3 != 0
            || p.indices.iter().any(|&i| i as usize >= n)
            || p.positions.iter().chain(&p.normals).any(|v| !v.is_finite())
            || p.base_color
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err("invalid authored actor primitive".into());
        }
        let base = mesh.positions.len() as u32;
        for point in p.positions.chunks_exact(3) {
            mesh.positions.push([point[0], point[1], point[2]]);
            mesh.uvs.push([0.0, 0.0]);
            mesh.colors.push(color);
        }
        for normal in p.normals.chunks_exact(3) {
            let normal = Vec3::new(normal[0], normal[1], normal[2]);
            if normal.length_squared() < 0.000_001 {
                return Err("zero authored actor normal".into());
            }
            mesh.normals.push(normal.normalize().to_array());
        }
        mesh.indices.extend(p.indices.into_iter().map(|i| base + i));
    }
    if mesh.positions.is_empty() {
        return Err("empty authored actor model".into());
    }
    let min = (0..3)
        .map(|a| {
            mesh.positions
                .iter()
                .map(|p| p[a])
                .fold(f32::INFINITY, f32::min)
        })
        .collect::<Vec<_>>();
    let max = (0..3)
        .map(|a| {
            mesh.positions
                .iter()
                .map(|p| p[a])
                .fold(f32::NEG_INFINITY, f32::max)
        })
        .collect::<Vec<_>>();
    if min[1].abs() > 0.000_1 || (0..3).any(|a| max[a] - min[a] < 0.005) {
        return Err("authored actor must be volumetric and floor rooted".into());
    }
    Ok(mesh)
}

pub(crate) fn gold_gym_boulder_mesh() -> SurfaceMeshData {
    static MODEL: OnceLock<SurfaceMeshData> = OnceLock::new();
    MODEL.get_or_init(|| parse_model_palette(
        crate::model_storage::include_model!("models/actor_props/boulder.mesh.json"), true,
    ).expect("named Cianwood boulder palette")).clone()
}

macro_rules! prop_mesh {
    ("icon_unown") => { crate::species_rig::rig(crate::species_rig::Species::Unown(8)).neutral.clone() };
    ("entei") => { crate::species_rig::rig(crate::species_rig::Species::Entei).neutral.clone() };
    ("raikou") => { crate::species_rig::rig(crate::species_rig::Species::Raikou).neutral.clone() };
    ("suicune") => { crate::species_rig::rig(crate::species_rig::Species::Suicune).neutral.clone() };
    ("slowpoke") => { crate::species_rig::rig(crate::species_rig::Species::Slowpoke).neutral.clone() };
    ("icon_slowpoke") => { crate::species_rig::rig(crate::species_rig::Species::Slowpoke).neutral.clone() };
    ("icon_diglett") => { crate::species_rig::rig(crate::species_rig::Species::Diglett).neutral.clone() };
    ("big_onix") => { crate::species_rig::rig(crate::species_rig::Species::Onix).neutral.clone() };
    ("icon_voltorb") => { crate::species_rig::rig(crate::species_rig::Species::Voltorb).neutral.clone() };
    ("icon_staryu") => { crate::species_rig::rig(crate::species_rig::Species::Staryu).neutral.clone() };
    ("icon_jigglypuff") => { crate::species_rig::rig(crate::species_rig::Species::Jigglypuff).neutral.clone() };
    ("icon_clefairy") => { crate::species_rig::rig(crate::species_rig::Species::Clefairy).neutral.clone() };
    ("chris_bike") => { crate::bicycle::mesh(false) };
    ("kris_bike") => { crate::bicycle::mesh(true) };
    ("biker") => { crate::bicycle::mesh(false) };
    ("battle_chikorita") => {
        crate::species_rig::rig(crate::species_rig::Species::Chikorita)
            .neutral
            .clone()
    };
    ("battle_cyndaquil") => {
        crate::species_rig::rig(crate::species_rig::Species::Cyndaquil)
            .neutral
            .clone()
    };
    ("battle_totodile") => {
        crate::species_rig::rig(crate::species_rig::Species::Totodile)
            .neutral
            .clone()
    };
    ($label:literal) => {{
        static MODEL: OnceLock<SurfaceMeshData> = OnceLock::new();
        MODEL
            .get_or_init(|| {
                parse_model(crate::model_storage::include_model!(concat!(
                    "models/actor_props/",
                    $label,
                    ".mesh.json"
                )))
                .expect(concat!("valid original actor asset: ", $label))
            })
            .clone()
    }};
}

macro_rules! authored_props {
    ($( $variant:ident => $label:tt ),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub(crate) enum PropKind { $( $variant ),+, ExactSpecies(&'static str) }
        pub(crate) const ALL_KINDS: &[PropKind] = &[$( PropKind::$variant ),+];
        impl PropKind {
            pub(crate) fn label(self) -> &'static str {
                match self { $( Self::$variant => $label ),+, Self::ExactSpecies(species) => species }
            }
        }
        pub(crate) fn mesh(kind: PropKind) -> SurfaceMeshData {
            match kind { $(
                PropKind::$variant => prop_mesh!($label)
            ),+ ,
                PropKind::ExactSpecies(species) => crate::battle_species_models::mesh(species)
                    .expect("verified exact visible species"),
            }
        }
    };
}
authored_props! {
    PokeBall => "poke_ball",
    Boulder => "boulder",
    Rock => "rock",
    FruitTree => "fruit_tree",
    Paper => "paper",
    Pokedex => "pokedex",
    Famicom => "famicom",
    Snes => "snes",
    N64 => "n64",
    VirtualBoy => "virtual_boy",
    GoldTrophy => "gold_trophy",
    SilverTrophy => "silver_trophy",
    Bird => "bird",
    Fairy => "fairy",
    Monster => "monster",
    Dragon => "dragon",
    Slowpoke => "slowpoke",
    Sudowoodo => "sudowoodo",
    Entei => "entei",
    Raikou => "raikou",
    Suicune => "suicune",
    BigSnorlax => "big_snorlax",
    BigLapras => "big_lapras",
    BigOnix => "big_onix",
    IconBat => "icon_bat",
    IconBigmon => "icon_bigmon",
    IconBird => "icon_bird",
    IconBlob => "icon_blob",
    IconBug => "icon_bug",
    IconBulbasaur => "icon_bulbasaur",
    IconCaterpillar => "icon_caterpillar",
    IconCharmander => "icon_charmander",
    IconClefairy => "icon_clefairy",
    IconDiglett => "icon_diglett",
    IconEgg => "icon_egg",
    IconEquine => "icon_equine",
    IconFighter => "icon_fighter",
    IconFish => "icon_fish",
    IconFox => "icon_fox",
    IconGeodude => "icon_geodude",
    IconGhost => "icon_ghost",
    IconGyarados => "icon_gyarados",
    IconHoOh => "icon_ho_oh",
    IconHumanshape => "icon_humanshape",
    IconJellyfish => "icon_jellyfish",
    IconJigglypuff => "icon_jigglypuff",
    IconLapras => "icon_lapras",
    IconLugia => "icon_lugia",
    IconMonster => "icon_monster",
    IconMoth => "icon_moth",
    IconOddish => "icon_oddish",
    IconPikachu => "icon_pikachu",
    IconPoliwag => "icon_poliwag",
    IconSerpent => "icon_serpent",
    IconShell => "icon_shell",
    IconSlowpoke => "icon_slowpoke",
    IconSnorlax => "icon_snorlax",
    ChrisBike => "chris_bike",
    KrisBike => "kris_bike",
    BikerBike => "biker",
    BattleChikorita => "battle_chikorita",
    BattleCyndaquil => "battle_cyndaquil",
    BattleTotodile => "battle_totodile",
    IconSquirtle => "icon_squirtle",
    IconStaryu => "icon_staryu",
    IconSudowoodo => "icon_sudowoodo",
    IconUnown => "icon_unown",
    IconVoltorb => "icon_voltorb",
}

impl PropKind {
    pub(crate) fn is_bicycle(self) -> bool {
        matches!(self, Self::ChrisBike | Self::KrisBike | Self::BikerBike)
    }
}

/// Exact resolved source/path handling. In particular, gfx/icons/monster.png
/// cannot accidentally claim the unrelated sprite family's representation.
pub(crate) fn prop_kind_for_source(source: &str) -> Option<PropKind> {
    // The publisher supplies only the currently resolved visible species and
    // actual bitmap source. Never recover a species from a shared icon family.
    if let Some(identity) = source.strip_prefix("species:") {
        let (species, art) = identity.split_once(':')?;
        if art.contains(':') || art.contains('/') {
            return None;
        }
        // The production publisher verifies these named NPCs against their
        // authored dialogue/cry; Crystal gives them shared species-named art.
        let shared_npc_art = matches!((species, art),
            ("DODRIO" | "PIDGEY" | "FEAROW" | "FARFETCH_D" | "MURKROW" | "SPEAROW", "moltres")
                | ("PSYDUCK" | "KANGASKHAN", "rhydon") | ("NIDORINO" | "NIDORAN_M" | "NIDORAN_F" | "PERSIAN" | "MEOWTH" | "RATTATA", "growlithe"));
        let known_art = art == species.to_ascii_lowercase()
            || prop_kind_for_source(art).is_some()
            || shared_npc_art;
        if !known_art {
            return None;
        }
        let exact = crate::battle_species_models::supported_species()
            .find(|candidate| *candidate == species)?;
        return Some(PropKind::ExactSpecies(exact));
    }
    let source = if let Some(path) = source.strip_prefix("gfx/sprites/") {
        let stem = path.strip_suffix(".png")?;
        if stem.starts_with("icon_") {
            return None;
        }
        stem.to_owned()
    } else if let Some(path) = source.strip_prefix("gfx/icons/") {
        format!("icon_{}", path.strip_suffix(".png")?)
    } else {
        source.to_owned()
    };
    if source.contains('/') || source.starts_with("battle_") {
        return None;
    }
    ALL_KINDS
        .iter()
        .copied()
        .find(|kind| kind.label() == source)
}

pub(crate) fn kind_labels() -> impl Iterator<Item = &'static str> {
    ALL_KINDS.iter().copied().map(PropKind::label)
}

/// Compatibility entry point for the previously supported orchard actor.
pub(super) fn fruit_tree() -> SurfaceMeshData {
    mesh(PropKind::FruitTree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn native_stone_paint_ignores_background_and_preserves_the_shared_shape() {
        use bevy::{prelude::Image,asset::RenderAssetUsages,render::render_resource::{Extent3d,TextureDimension,TextureFormat}};
        let mut image=Image::new_fill(Extent3d {width:4,height:4,depth_or_array_layers:1},
            TextureDimension::D2,&[0,0,0,0],TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::default());
        let data=image.data.as_mut().unwrap();
        for p in data[..24].chunks_exact_mut(4) {p.copy_from_slice(&[165,123,24,255]);}
        for p in data[24..].chunks_exact_mut(4) {p.copy_from_slice(&[57,57,57,255]);}
        assert_eq!(native_stone_color(&image),Some([165,123,24]));
        for kind in [PropKind::Rock,PropKind::Boulder] {
            let old=mesh(kind);let painted=native_stone_mesh(kind,[165,123,24]);
            assert_eq!(painted.positions,old.positions);assert_eq!(painted.normals,old.normals);
            assert_eq!(painted.indices,old.indices);
            assert!(painted.colors.iter().all(|c|c[0]>c[1] && c[1]>c[2]));
            assert!(painted.uvs.iter().all(|uv|crate::scenery_finish::source_atlas_coordinates(*uv).is_some()));
            assert!(painted.uvs.iter().any(|uv|uv!=&painted.uvs[0]),"stone grain spans the closed faces");
        }
        let pixels=image.data.as_mut().unwrap();
        for p in pixels[..40].chunks_exact_mut(4) {p.copy_from_slice(&[66,33,41,255]);}
        for p in pixels[40..].chunks_exact_mut(4) {p.copy_from_slice(&[99,74,123,255]);}
        assert_eq!(native_stone_color(&image),Some([99,74,123]),
            "the native night highlight must remain visible on the fractured faces");
        for p in image.data.as_mut().unwrap().chunks_exact_mut(4) {p.copy_from_slice(&[66,33,41,255]);}
        assert_eq!(native_stone_color(&image),Some([66,33,41]),"do not invent an absent highlight");
        image.data.as_mut().unwrap().fill(0);
        assert_eq!(native_stone_color(&image),None);
    }

    #[test]
    fn smashable_stone_is_one_closed_fractured_volume_inside_its_source_cell() {
        let rock=mesh(PropKind::Rock);
        let low=rock.positions.iter().fold(Vec3::splat(f32::INFINITY),|a,p|a.min(Vec3::from_array(*p)));
        let high=rock.positions.iter().fold(Vec3::splat(f32::NEG_INFINITY),|a,p|a.max(Vec3::from_array(*p)));
        assert!((low-Vec3::new(-0.32,0.,-0.27)).abs().max_element()<0.000002);
        assert!((high-Vec3::new(0.32,0.43,0.27)).abs().max_element()<0.000002);
        let point=|i:u32|rock.positions[i as usize].map(|v|(v*1_000_000.).round() as i32);
        let mut edges=std::collections::BTreeMap::new();
        let mut adjacency=std::collections::BTreeMap::<[i32;3],Vec<[i32;3]>>::new();
        let mut volume=0.;
        for tri in rock.indices.chunks_exact(3) {
            let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(rock.positions[i as usize]));
            let normal=(b-a).cross(c-a);
            assert!(normal.length_squared()>0.000000000001);
            volume+=a.dot(b.cross(c))/6.;
            for i in 0..3 {
                let (a,b)=(point(tri[i]),point(tri[(i+1)%3]));
                let edge=if a<b {(a,b)}else{(b,a)};
                *edges.entry(edge).or_insert(0usize)+=1;
                adjacency.entry(a).or_default().push(b);
            }
        }
        assert!(edges.values().all(|count|*count==2),"no cracks or overlapping shells");
        assert!(volume>0.01,"outward winding encloses a real stone");
        let mut queue=vec![*adjacency.keys().next().unwrap()];let mut seen=HashSet::new();
        while let Some(p)=queue.pop() {if seen.insert(p) {queue.extend_from_slice(&adjacency[&p]);}}
        assert_eq!(seen.len(),adjacency.len(),"one stone, no detached lumps");
        assert_eq!(rock.indices.len()/3,98);
        for rgb in [[165,123,24],[120,80,160]] {
            let finish=native_stone_mesh(PropKind::Rock,rgb);
            assert_eq!(finish.positions,rock.positions);assert_eq!(finish.indices,rock.indices);
            assert!(finish.colors.iter().all(|c|c.iter().all(|v|v.is_finite()) && c[3]==1.));
        }
    }

    #[test]
    fn gold_gym_boulder_named_palette_preserves_geometry_and_other_assets() {
        let ordinary = mesh(PropKind::Boulder);
        let gym = gold_gym_boulder_mesh();
        assert_eq!(ordinary.positions, gym.positions);
        assert_eq!(ordinary.normals, gym.normals);
        assert_eq!(ordinary.indices, gym.indices);
        assert_eq!(ordinary.uvs, gym.uvs);
        assert_ne!(ordinary.colors, gym.colors);
        assert_eq!(mesh(PropKind::Boulder), ordinary);
        let json = include_str!("../models/actor_props/boulder.mesh.json");
        assert!(parse_model_palette(json.replace("stone_light", "unknown").as_str(), true).is_err());
        assert!(parse_model_palette(json.replace("\"boulder\"", "\"rock\"").as_str(), true).is_err());
        // These are the reviewed sprite palette's ochre body, gold highlight
        // and charcoal outline. Comparing decoded display colors catches a
        // generic gray substitution as well as double sRGB conversion.
        let display_rgb = |color: &[f32; 4]| -> [u8; 3] {
            std::array::from_fn(|i| {
                let v = color[i];
                let srgb = if v <= 0.0031308 { v * 12.92 }
                    else { 1.055 * v.powf(1. / 2.4) - 0.055 };
                (srgb * 255.).round() as u8
            })
        };
        let expected = [[165, 123, 24], [198, 148, 57], [57, 57, 57]];
        for rgb in expected {
            assert!(gym.colors.iter().any(|color| display_rgb(color) == rgb),
                "movable Gym boulders must retain every source palette shade");
        }
        for color in &gym.colors {
            assert!(color[3] == 1. && color.iter().all(|v| v.is_finite()));
            assert!(expected.contains(&display_rgb(color)),
                "movable Gym boulders must not inherit a generic stone palette");
        }
    }

    #[test]
    fn exact_source_families_preserve_icon_namespace_and_reject_guesses() {
        assert_eq!(prop_kind_for_source("poke_ball"), Some(PropKind::PokeBall));
        assert_eq!(
            prop_kind_for_source("gfx/sprites/monster.png"),
            Some(PropKind::Monster)
        );
        assert_eq!(
            prop_kind_for_source("gfx/icons/monster.png"),
            Some(PropKind::IconMonster)
        );
        assert_ne!(
            prop_kind_for_source("monster"),
            prop_kind_for_source("icon_monster")
        );
        for bad in [
            "surf",
            "surfing_pikachu",
            "gfx/sprites/surf.png",
            "gfx/sprites/surfing_pikachu.png",
            "fruit_tree_picked",
            "monster_extra",
            "SPRITE_MONSTER",
            "pikachu",
            "remote_player",
            "battle_chikorita",
            "gfx/sprites/icon_monster.png/extra",
            "gfx/sprites/icon_monster.png",
            "path/poke_ball",
        ] {
            assert_eq!(prop_kind_for_source(bad), None, "{bad}");
        }
        for kind in ALL_KINDS
            .iter()
            .filter(|kind| !kind.label().starts_with("battle_"))
        {
            assert_eq!(prop_kind_for_source(kind.label()), Some(*kind));
        }
    }

    #[test]
    fn exact_species_props_share_canonical_bodies_without_icon_substitution() {
        for species in ["CHIKORITA", "CYNDAQUIL", "TOTODILE", "PIDGEY", "CLEFAIRY", "JIGGLYPUFF", "RATTATA"] {
            assert_eq!(mesh(PropKind::ExactSpecies(species)),
                crate::battle_species_models::mesh(species).unwrap());
            assert!(crate::has_authored_battle_species(species));
        }
        for source in ["monster", "icon_fish", "species:UNKNOWN:icon_fish"] {
            assert!(!crate::has_authored_battle_species(source));
        }
        assert_eq!(prop_kind_for_source("species:UNKNOWN:icon_fish"),None);
        for (species, icon, kind) in [("STARYU", "icon_staryu", PropKind::IconStaryu), ("VOLTORB", "icon_voltorb", PropKind::IconVoltorb), ("ONIX", "big_onix", PropKind::BigOnix), ("DIGLETT", "icon_diglett", PropKind::IconDiglett), ("SLOWPOKE", "slowpoke", PropKind::Slowpoke), ("SLOWPOKE", "icon_slowpoke", PropKind::IconSlowpoke), ("UNOWN", "icon_unown", PropKind::IconUnown), ("ENTEI", "entei", PropKind::Entei), ("RAIKOU", "raikou", PropKind::Raikou), ("SUICUNE", "suicune", PropKind::Suicune)] {
            assert!(crate::has_authored_battle_species(species));
            assert_eq!(prop_kind_for_source(&format!("species:{species}:{icon}")), Some(PropKind::ExactSpecies(species)));
            assert_eq!(mesh(kind), mesh(PropKind::ExactSpecies(species)));
        }
    }

    #[test]
    fn every_authored_actor_is_finite_volumetric_and_floor_rooted() {
        let mut labels = HashSet::new();
        for &kind in ALL_KINDS {
            assert!(labels.insert(kind.label()));
            let model = mesh(kind);
            assert!(model.positions.len() >= 12, "{}", kind.label());
            assert_eq!(model.positions.len(), model.normals.len());
            assert_eq!(model.positions.len(), model.colors.len());
            assert_eq!(model.positions.len(), model.uvs.len());
            let min_y = model
                .positions
                .iter()
                .map(|p| p[1])
                .fold(f32::INFINITY, f32::min);
            assert!(min_y.abs() < 0.0001, "{}: {min_y}", kind.label());
            assert!(model
                .indices
                .iter()
                .all(|&i| (i as usize) < model.positions.len()));
            for n in model.normals {
                assert!((Vec3::from_array(n).length() - 1.0).abs() < 0.0001);
            }
        }
        // Registry size changes as retired sprite props move to shared rigs.
        // Validate the current registry above rather than requiring old assets.
        assert!(!labels.contains("surf") && !labels.contains("surfing_pikachu"));
    }

    #[test]
    fn malformed_or_unanchored_exports_fail_closed() {
        assert!(parse_model(r#"{"primitives":[]}"#).is_err());
        assert!(parse_model(r#"{"primitives":[{"positions":[0,0,0],"normals":[0,1,0],"indices":[0,1,2],"base_color":[1,1,1,1]}]}"#).is_err());
    }
}

#[cfg(test)]
mod exact_visible_species_tests {
    use super::*;
    #[test]
    fn ho_oh_uses_its_dedicated_battle_model_and_preserves_icon_routing() {
        assert_eq!(
            crate::battle_species_models::supported_species()
                .filter(|name| *name == "HO_OH")
                .count(),
            1
        );
        assert_eq!(prop_kind_for_source("icon_ho_oh"), Some(PropKind::IconHoOh));
        assert_eq!(
            prop_kind_for_source("species:HO_OH:icon_ho_oh"),
            Some(PropKind::ExactSpecies("HO_OH"))
        );
        let model = mesh(PropKind::ExactSpecies("HO_OH"));
        let battle = crate::battle_species_models::mesh("HO_OH").unwrap();
        assert_eq!(model.positions, battle.positions);
        assert_eq!(model.indices, battle.indices);
        assert_eq!(model.normals, battle.normals);
        assert_eq!(model.colors, battle.colors);
        assert_ne!(model.positions, mesh(PropKind::IconHoOh).positions);
    }

    #[test]
    fn lugia_uses_its_dedicated_battle_model_and_preserves_icon_routing() {
        assert_eq!(
            crate::battle_species_models::supported_species()
                .filter(|name| *name == "LUGIA")
                .count(),
            1
        );
        assert_eq!(prop_kind_for_source("icon_lugia"), Some(PropKind::IconLugia));
        assert_eq!(
            prop_kind_for_source("species:LUGIA:icon_lugia"),
            Some(PropKind::ExactSpecies("LUGIA"))
        );
        let model = mesh(PropKind::ExactSpecies("LUGIA"));
        let battle = crate::battle_species_models::mesh("LUGIA").unwrap();
        assert_eq!(model.positions, battle.positions);
        assert_eq!(model.indices, battle.indices);
        assert_eq!(model.normals, battle.normals);
        assert_eq!(model.colors, battle.colors);
        assert_ne!(model.positions, mesh(PropKind::IconLugia).positions);
    }

    #[test]
    fn published_named_pokemon_use_their_species_with_shared_bitmap() {
        for (species, art) in [
            ("DODRIO", "moltres"),
            ("PIDGEY", "moltres"),
            ("FEAROW", "moltres"),
            ("PSYDUCK", "rhydon"),
            ("KANGASKHAN", "rhydon"),
            ("NIDORINO", "growlithe"),
            ("RATTATA", "growlithe"),
            ("FARFETCH_D", "moltres"),
            ("MURKROW", "moltres"),
            ("SPEAROW", "moltres"),
            ("NIDORAN_M", "growlithe"),
            ("NIDORAN_F", "growlithe"),
            ("PERSIAN", "growlithe"),
            ("MEOWTH", "growlithe"),
        ] {
            let kind = prop_kind_for_source(&format!("species:{species}:{art}")).unwrap();
            assert_eq!(kind, PropKind::ExactSpecies(species));
            let model = mesh(kind);
            let expected = crate::battle_species_models::mesh(species).unwrap();
            assert_eq!(model.positions, expected.positions);
            assert_eq!(model.indices, expected.indices);
        }
        assert_eq!(prop_kind_for_source("species:PIKACHU:moltres"), None);
        assert_eq!(prop_kind_for_source("moltres"), None);
    }

    #[test]
    fn published_amphy_identity_reuses_exact_ampharos_mesh_and_keeps_monsters_generic() {
        let kind = prop_kind_for_source("species:AMPHAROS:monster").unwrap();
        assert_eq!(kind, PropKind::ExactSpecies("AMPHAROS"));
        assert_eq!(prop_kind_for_source("monster"), Some(PropKind::Monster));
        assert_eq!(
            prop_kind_for_source("icon_monster"),
            Some(PropKind::IconMonster)
        );
        let model = mesh(kind);
        let exact = crate::battle_species_models::mesh("AMPHAROS").unwrap();
        assert_eq!(model.positions, exact.positions);
        assert_eq!(model.indices, exact.indices);
        assert_eq!(model.normals, exact.normals);
        assert_eq!(model.colors, exact.colors);
        let min_y = model
            .positions
            .iter()
            .map(|point| point[1])
            .fold(f32::INFINITY, f32::min);
        assert!(min_y.abs() < 0.0001);
    }

    #[test]
    fn shared_icon_families_do_not_erase_current_species() {
        for (species, art) in [
            ("MAGIKARP", "icon_fish"),
            ("GENGAR", "icon_ghost"),
            ("GRIMER", "icon_blob"),
            ("WEEDLE", "icon_caterpillar"),
            ("MACHOP", "icon_humanshape"),
            ("TENTACOOL", "icon_jellyfish"),
            ("SHELLDER", "icon_shell"),
        ] {
            let kind = prop_kind_for_source(&format!("species:{species}:{art}")).unwrap();
            assert_eq!(kind, PropKind::ExactSpecies(species));
            assert_ne!(kind, prop_kind_for_source(art).unwrap());
            let sculpt = mesh(kind);
            assert!(!sculpt.indices.is_empty());
            assert!(sculpt.positions.iter().any(|p| p[2] > 0.01));
        }
        for unknown in [
            "species:MISSING:icon_fish",
            "species:MAGIKARP:new_art",
            "species:MAGIKARP:gfx/icons/fish.png",
            "species:MAGIKARP:species:GENGAR:icon_ghost",
        ] {
            assert_eq!(prop_kind_for_source(unknown), None, "{unknown}");
        }
    }
    #[test]
    fn gyarados_uses_the_dedicated_serpent_and_keeps_generic_icon_routing() {
        assert_eq!(prop_kind_for_source("species:GYARADOS:icon_gyarados"), Some(PropKind::ExactSpecies("GYARADOS")));
        assert_eq!(prop_kind_for_source("icon_gyarados"), Some(PropKind::IconGyarados));
        let exact = mesh(PropKind::ExactSpecies("GYARADOS"));
        let battle = crate::battle_species_models::mesh("GYARADOS").unwrap();
        assert_eq!(exact.positions, battle.positions);
        assert_eq!(exact.indices, battle.indices);
    }

    #[test]
    fn sudowoodo_exact_source_shares_battle_body_and_preserves_legacy_source() {
        assert_eq!(prop_kind_for_source("species:SUDOWOODO:sudowoodo"), Some(PropKind::ExactSpecies("SUDOWOODO")));
        assert_eq!(prop_kind_for_source("sudowoodo"), Some(PropKind::Sudowoodo));
        let visible = mesh(PropKind::ExactSpecies("SUDOWOODO"));
        let battle = crate::battle_species_models::mesh("SUDOWOODO").unwrap();
        assert_eq!(visible.positions, battle.positions);
        assert_eq!(visible.indices, battle.indices);
    }

    #[test]
    fn every_canonical_species_has_a_bounded_reusable_visible_model() {
        let mut species: Vec<_> = crate::battle_species_models::supported_species()
            .collect();
        species.sort_unstable();
        species.dedup();
        assert_eq!(species.len(), 251);
        for name in species {
            let source = format!("species:{name}:icon_monster");
            assert_eq!(
                prop_kind_for_source(&source),
                Some(PropKind::ExactSpecies(name))
            );
            let model = mesh(PropKind::ExactSpecies(name));
            assert_eq!(model.positions.len(), model.normals.len(), "{name}");
            assert_eq!(model.positions.len(), model.colors.len(), "{name}");
            assert!(model.indices.len() >= 60, "{name}");
            assert!(
                model
                    .indices
                    .iter()
                    .all(|&i| (i as usize) < model.positions.len()),
                "{name}"
            );
            assert!(
                model.positions.iter().flatten().all(|v| v.is_finite()),
                "{name}"
            );
            let lo = std::array::from_fn::<_, 3, _>(|axis| {
                model
                    .positions
                    .iter()
                    .map(|p| p[axis])
                    .fold(f32::INFINITY, f32::min)
            });
            let hi = std::array::from_fn::<_, 3, _>(|axis| {
                model
                    .positions
                    .iter()
                    .map(|p| p[axis])
                    .fold(f32::NEG_INFINITY, f32::max)
            });
            assert!(lo[1].abs() < 0.0001, "{name}");
            assert!((0..3).all(|axis| hi[axis] - lo[axis] > 0.005), "{name}");
            assert!(
                model.normals.iter().all(|n| n.iter().all(|v| v.is_finite())
                    && (n.iter().map(|v| v * v).sum::<f32>() - 1.0).abs() < 0.0001),
                "{name}"
            );
        }
    }
}
