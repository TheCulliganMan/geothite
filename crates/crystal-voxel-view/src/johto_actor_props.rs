//! Original source-scoped volumetric actor assets, with no simulation state.
//!
//! Runtime source identity is published after variable-sprite and icon
//! resolution. `monster` and `icon_monster` therefore remain different keys.
//! Models are ground rooted (+Y up, front +Z) and never infer species from
//! colors, original script tokens, actor IDs, or another actor's appearance.
use std::sync::OnceLock;

use crate::mesh::SurfaceMeshData;
use crate::species_rig::{self, Species};
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

// Canonical geometry can depict either a living creature or an inert display.
// This metadata is presentation-only; source identity still comes from the publisher.
macro_rules! prop_model {
    () => { None };
    ([$role:ident $species:expr $(, $height:literal)?]) => { Some(species_rig::rig($species)) };
}
macro_rules! prop_display_height {
    () => { None };
    ([rig $species:expr]) => { None };
    ([display $species:expr, $height:literal]) => { Some($height) };
}
macro_rules! prop_mesh {
    ($label:tt, [$role:ident $species:expr $(, $height:literal)?]) => {
        species_rig::rig($species).neutral.clone()
    };
    ("chris_bike") => { crate::bicycle::mesh(false) };
    ("kris_bike") => { crate::bicycle::mesh(true) };
    ("biker") => { crate::bicycle::mesh(false) };
    ($label:literal) => {{
        static MODEL: OnceLock<SurfaceMeshData> = OnceLock::new();
        MODEL.get_or_init(|| {
            parse_model(crate::model_storage::include_model!(concat!(
                "models/actor_props/", $label, ".mesh.json"
            ))).expect(concat!("valid original actor asset: ", $label))
        }).clone()
    }};
}

macro_rules! authored_props {
    ($( $variant:ident => $label:tt $( [$role:ident $species:expr $(, $height:literal)?] )? ),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub(crate) enum PropKind { $( $variant ),+, ExactSpecies(&'static str), ExactDisplay(&'static str), SurfingPikachuDisplay }
        pub(crate) const ALL_KINDS: &[PropKind] = &[$( PropKind::$variant ),+];
        impl PropKind {
            pub(crate) fn label(self) -> &'static str {
                match self { $( Self::$variant => $label ),+, Self::ExactSpecies(species) | Self::ExactDisplay(species) => species,
                    Self::SurfingPikachuDisplay => "surfing_pikachu_doll" }
            }
            /// The visual template never establishes a gameplay species identity.
            pub(crate) fn pokemon_model(self) -> Option<&'static species_rig::SpeciesRig> {
                match self {
                    $( Self::$variant => prop_model!($([$role $species $(, $height)?])?), )+
                    Self::ExactSpecies(name) | Self::ExactDisplay(name) => species_rig::for_species(name),
                    Self::SurfingPikachuDisplay => Some(species_rig::rig(Species::Pikachu)),
                }
            }
            pub(crate) fn display_height(self) -> Option<f32> {
                match self {
                    $( Self::$variant => prop_display_height!($([$role $species $(, $height)?])?), )+
                    Self::ExactSpecies(_) => None,
                    Self::ExactDisplay(_) => Some(0.82),
                    Self::SurfingPikachuDisplay => Some(0.82),
                }
            }
            pub(crate) fn pokemon_rig(self) -> Option<&'static species_rig::SpeciesRig> {
                self.display_height().is_none().then(|| self.pokemon_model()).flatten()
            }
            pub(crate) fn display_scale(self) -> f32 {
                self.display_height().map_or(1., |height| {
                    let rig=self.pokemon_model().expect("named display model");
                    let board = if self == Self::SurfingPikachuDisplay { SURFBOARD_DECK_Y } else { 0. };
                    height / (rig.neutral_bounds.1.y-rig.neutral_bounds.0.y + board)
                })
            }
        }
        pub(crate) fn mesh(kind: PropKind) -> SurfaceMeshData {
            match kind { $(
                PropKind::$variant => prop_mesh!($label $(, [$role $species $(, $height)?])?)
            ),+ ,
                PropKind::ExactSpecies(species) | PropKind::ExactDisplay(species) => crate::battle_species_models::mesh(species)
                    .expect("verified exact visible species"),
                PropKind::SurfingPikachuDisplay => surfing_pikachu_display(),
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
    Slowpoke => "slowpoke" [rig Species::Slowpoke],
    Sudowoodo => "sudowoodo" [rig Species::Sudowoodo],
    Entei => "entei" [rig Species::Entei],
    Raikou => "raikou" [rig Species::Raikou],
    Suicune => "suicune" [rig Species::Suicune],
    BigSnorlax => "big_snorlax" [display Species::Snorlax, 1.48],
    BigLapras => "big_lapras" [display Species::Lapras, 1.52],
    BigOnix => "big_onix" [display Species::Onix, 1.72],
    IconBat => "icon_bat",
    IconBigmon => "icon_bigmon",
    IconBird => "icon_bird",
    IconBlob => "icon_blob",
    IconBug => "icon_bug",
    IconBulbasaur => "icon_bulbasaur" [display Species::Bulbasaur, 0.82],
    IconCaterpillar => "icon_caterpillar",
    IconCharmander => "icon_charmander" [display Species::Charmander, 0.82],
    IconClefairy => "icon_clefairy" [display Species::Clefairy, 0.82],
    IconDiglett => "icon_diglett" [display Species::Diglett, 0.82],
    IconEgg => "icon_egg",
    IconEquine => "icon_equine",
    IconFighter => "icon_fighter",
    IconFish => "icon_fish",
    IconFox => "icon_fox",
    IconGeodude => "icon_geodude" [display Species::Geodude, 0.82],
    IconGhost => "icon_ghost",
    IconGyarados => "icon_gyarados" [display Species::Gyarados, 0.82],
    IconHoOh => "icon_ho_oh" [display Species::HoOh, 0.82],
    IconHumanshape => "icon_humanshape",
    IconJellyfish => "icon_jellyfish",
    IconJigglypuff => "icon_jigglypuff" [display Species::Jigglypuff, 0.82],
    IconLapras => "icon_lapras" [display Species::Lapras, 0.82],
    IconLugia => "icon_lugia" [display Species::Lugia, 0.82],
    IconMonster => "icon_monster",
    IconMoth => "icon_moth",
    IconOddish => "icon_oddish" [display Species::Oddish, 0.82],
    IconPikachu => "icon_pikachu" [display Species::Pikachu, 0.82],
    IconPoliwag => "icon_poliwag" [display Species::Poliwag, 0.82],
    IconSerpent => "icon_serpent",
    IconShell => "icon_shell",
    IconSlowpoke => "icon_slowpoke" [display Species::Slowpoke, 0.82],
    IconSnorlax => "icon_snorlax" [display Species::Snorlax, 0.82],
    ChrisBike => "chris_bike",
    KrisBike => "kris_bike",
    BikerBike => "biker",
    BattleChikorita => "battle_chikorita" [rig Species::Chikorita],
    BattleCyndaquil => "battle_cyndaquil" [rig Species::Cyndaquil],
    BattleTotodile => "battle_totodile" [rig Species::Totodile],
    IconSquirtle => "icon_squirtle" [display Species::Squirtle, 0.82],
    IconStaryu => "icon_staryu" [display Species::Staryu, 0.82],
    IconSudowoodo => "icon_sudowoodo" [display Species::Sudowoodo, 0.82],
    IconUnown => "icon_unown" [display Species::Unown(8), 0.82],
    IconVoltorb => "icon_voltorb" [display Species::Voltorb, 0.82],
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
    let identity=source.strip_prefix("species:").map(|id|(id,false))
        .or_else(||source.strip_prefix("display:").map(|id|(id,true)));
    if let Some((identity, display)) = identity {
        let (species, art) = identity.split_once(':')?;
        // Only the explicitly published bedroom ornament receives a board.
        // The raw Surf avatar and actual mounted Pikachu keep their own paths.
        if display && species == "PIKACHU" && art == "surfing_pikachu" {
            return Some(PropKind::SurfingPikachuDisplay);
        }
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
        if display {
            // Named big dolls keep their authored size. A shared small icon
            // still depicts the explicitly published species, never its family.
            if let Some(kind)=prop_kind_for_source(art) {
                if kind.display_height().is_some()
                    && kind.pokemon_model().is_some_and(|rig|
                        Some(rig.species)==species_rig::for_species(exact).map(|r|r.species)) {
                    return Some(kind);
                }
            }
            return Some(PropKind::ExactDisplay(exact));
        }
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

const SURFBOARD_DECK_Y: f32 = 0.045;

/// An inert ornament assembled from the unchanged canonical mouse and a
/// separate closed, bevelled board. No duplicate Pokémon asset or rider rig.
fn surfing_pikachu_display() -> SurfaceMeshData {
    static MODEL: OnceLock<SurfaceMeshData> = OnceLock::new();
    MODEL.get_or_init(|| {
        let mut mesh = species_rig::rig(Species::Pikachu).neutral.clone();
        for point in &mut mesh.positions { point[1] += SURFBOARD_DECK_Y; }
        let deck = [0.045, 0.25, 0.38, 1.];
        let rail = [0.85, 0.88, 0.78, 1.];
        let segments = 40;
        let point = |i: usize, y: f32, radius: f32| {
            let angle = i as f32 / segments as f32 * std::f32::consts::TAU;
            Vec3::new(0.52 * radius * angle.cos(), y, 0.24 * radius * angle.sin())
        };
        let mut triangle = |points: [Vec3; 3], color: [f32; 4]| {
            let normal = (points[1]-points[0]).cross(points[2]-points[0]).normalize();
            let start = mesh.positions.len() as u32;
            for point in points {
                mesh.positions.push(point.to_array());
                mesh.normals.push(normal.to_array());
                mesh.colors.push(color);
                mesh.uvs.push([0.,0.]);
            }
            mesh.indices.extend([start, start+1, start+2]);
        };
        for i in 0..segments {
            let j = (i+1)%segments;
            triangle([Vec3::Y*SURFBOARD_DECK_Y,
                point(j,SURFBOARD_DECK_Y,0.95),point(i,SURFBOARD_DECK_Y,0.95)],deck);
            triangle([Vec3::ZERO,point(i,0.,0.95),point(j,0.,0.95)],rail);
            for (lo,hi,a,b) in [(0.,SURFBOARD_DECK_Y*0.5,0.95,1.),
                (SURFBOARD_DECK_Y*0.5,SURFBOARD_DECK_Y,1.,0.95)] {
                triangle([point(i,lo,a),point(i,hi,b),point(j,hi,b)],rail);
                triangle([point(i,lo,a),point(j,hi,b),point(j,lo,a)],rail);
            }
        }
        mesh
    }).clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn surfing_pikachu_doll_keeps_canonical_body_and_closed_grounded_board() {
        let kind=prop_kind_for_source("display:PIKACHU:surfing_pikachu").unwrap();
        assert_eq!(kind,PropKind::SurfingPikachuDisplay);
        assert!(kind.pokemon_rig().is_none());
        let body=&species_rig::rig(Species::Pikachu).neutral;
        let display=mesh(kind);let n=body.positions.len();
        assert_eq!(&display.normals[..n],&body.normals);
        assert_eq!(&display.colors[..n],&body.colors);
        assert_eq!(&display.uvs[..n],&body.uvs);
        assert_eq!(&display.indices[..body.indices.len()],&body.indices);
        for (actual,original) in display.positions.iter().zip(&body.positions) {
            assert_eq!(*actual,[original[0],original[1]+SURFBOARD_DECK_Y,original[2]]);
        }
        let board=&display.positions[n..];
        assert_eq!(board.iter().map(|p|p[1]).fold(f32::INFINITY,f32::min),0.);
        assert_eq!(board.iter().map(|p|p[1]).fold(0.,f32::max),SURFBOARD_DECK_Y);
        let mut edges=std::collections::BTreeMap::new();let mut volume=0.;
        for tri in display.indices[body.indices.len()..].chunks_exact(3) {
            let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(display.positions[i as usize]));
            let normal=(points[1]-points[0]).cross(points[2]-points[0]);
            assert!(normal.is_finite() && normal.length_squared()>1e-14);
            assert!(normal.normalize().dot(Vec3::from_array(display.normals[tri[0] as usize]))>0.999);
            volume+=points[0].dot(points[1].cross(points[2]))/6.;
            let keys=points.map(|p|p.to_array().map(f32::to_bits));
            for (a,b) in [(0,1),(1,2),(2,0)] {
                let edge=if keys[a]<keys[b] {(keys[a],keys[b])}else{(keys[b],keys[a])};
                *edges.entry(edge).or_insert(0)+=1;
            }
        }
        assert!(volume>0.01,"board winding encloses solid volume");
        assert!(edges.values().all(|&count|count==2),"closed board has no open or duplicate edges");
        let high=display.positions.iter().map(|p|p[1]).fold(0.,f32::max);
        assert!((high*kind.display_scale()-0.82).abs()<1e-5);
        for source in ["surfing_pikachu","gfx/sprites/surfing_pikachu.png",
            "species:PIKACHU:surfing_pikachu","display:RAICHU:surfing_pikachu"] {
            assert_eq!(prop_kind_for_source(source),None,"{source} must not become a doll");
        }
    }

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
    fn named_pokemon_templates_share_canonical_geometry_and_keep_display_sizes() {
        let mut count=0;
        for &kind in ALL_KINDS {
            if let Some(rig)=kind.pokemon_model() {
                count+=1;
                assert_eq!(mesh(kind),rig.neutral,"{}",kind.label());
                if let Some(height)=kind.display_height() {
                    assert!(kind.pokemon_rig().is_none(),"inert display: {}",kind.label());
                    let span=rig.neutral_bounds.1.y-rig.neutral_bounds.0.y;
                    assert!((span*kind.display_scale()-height).abs()<0.000001);
                } else {
                    assert!(std::ptr::eq(kind.pokemon_rig().unwrap(),rig));
                }
            }
        }
        assert_eq!(count,31);
        for (kind,height) in [(PropKind::BigLapras,1.52),(PropKind::BigSnorlax,1.48),
            (PropKind::IconLapras,0.82),(PropKind::IconSnorlax,0.82),(PropKind::BigOnix,1.72)] {
            assert_eq!(kind.display_height(),Some(height));
            assert_eq!(prop_kind_for_source(kind.label()),Some(kind));
        }
        for kind in [PropKind::IconBird,PropKind::IconFish,PropKind::Monster,
            PropKind::PokeBall,PropKind::Boulder] {
            assert!(kind.pokemon_model().is_none(),"do not guess a species from {}",kind.label());
        }
        let display=prop_kind_for_source("display:BLASTOISE:icon_squirtle").unwrap();
        assert_eq!(display,PropKind::ExactDisplay("BLASTOISE"));
        assert_eq!(display.pokemon_model().unwrap().species,Species::Blastoise);
        assert!(display.pokemon_rig().is_none());
        assert_eq!(mesh(display),mesh(PropKind::ExactSpecies("BLASTOISE")));
        assert_eq!(prop_kind_for_source("display:LAPRAS:big_lapras"),Some(PropKind::BigLapras));
        assert_eq!(prop_kind_for_source("display:ONIX:big_onix"),Some(PropKind::BigOnix));
        assert_eq!(prop_kind_for_source("display:UNKNOWN:icon_fish"),None);
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
