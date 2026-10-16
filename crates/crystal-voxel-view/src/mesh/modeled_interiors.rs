//! Complete-source furnishings, independent of wall pixels and collision data.
//!
//! Resolve against immutable source cells before any legacy/live masking. Every
//! reserved object has a verified ground sample. Unknown, altered and clipped
//! drawings remain unclaimed and retain the existing rendering path.
use super::*;
use crate::interior_models::{ModelKind, model};
use crate::live_profiles::{Document, Object};
use std::sync::OnceLock;

#[path = "room_perimeter.rs"]
mod room_perimeter;
pub(super) use room_perimeter::append as append_room_perimeter;

#[derive(Clone, Debug)]
pub(super) struct Placement {
    pub(super) kind: ModelKind,
    pub(super) column: usize,
    pub(super) row: usize,
    pub(super) width: usize,
    pub(super) height: usize,
    ground: usize,
    depth_pixels: f32,
    height_pixels: f32,
    base_pixels: f32,
    footing_pixels: Option<f32>,
}
impl Placement {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        (0..self.height).flat_map(move |y| {
            (0..self.width).map(move |x| (self.row + y) * width + self.column + x)
        })
    }
    pub(super) fn seat_height_pixels(&self, map: &str, source: &VisualTileSource) -> Option<f32> {
        if self.kind==ModelKind::Chair && crate::pokecom::chair_local(map,source).is_some() {
            return Some(7.);
        }
        if map=="EarlsPokemonAcademy"&&self.kind==ModelKind::LinkSeat
            &&source.tileset_id.as_ref()=="lab"&&source.metatile_id==0x16 {return Some(5.2);}
        if map == "CeladonCafe" && self.kind == ModelKind::ArcadeStool
            && source.tileset_id.as_ref() == "game_corner" && matches!(source.metatile_id, 5 | 6 | 0x19) {
            return Some(5.);
        }
        if kanto_native_floor_map(map) && map.ends_with("Pokecenter1F") && self.kind == ModelKind::LinkSeat
            && source.tileset_id.as_ref() == "pokecenter" && matches!(source.metatile_id, 0x2e | 0x2f) {
            return Some(7.);
        }
        if self.kind == ModelKind::Cushion && source.tileset_id.as_ref() == "mart" {
            return Some(7.);
        }
        if self.kind == ModelKind::Stool && (reviewed_native_room(map) || map == "CeladonCafe")
            && crate::house::furniture_local(source).is_some_and(|(_, _, kind)| kind == crate::house::FurnitureKind::Stool) {
            return Some(5.);
        }
        if !matches!(self.kind, ModelKind::Stool | ModelKind::ArcadeStool
            | ModelKind::Chair | ModelKind::LinkSeat | ModelKind::StationBench) { return None; }
        let fraction = crate::interior_models::selected_model(self.kind)?.seat_height_fraction();
        Some(self.base_pixels + self.height_pixels * fraction)
    }
    pub(super) fn label(&self) -> &'static str {
        self.kind.label()
    }
}
#[path = "native_clinic.rs"]
mod native_clinic;
#[path = "native_pokecom_seats.rs"]
mod native_pokecom_seats;
#[path = "native_pokecom_equipment.rs"]
mod native_pokecom_equipment;
#[path = "native_clinic_fixtures.rs"]
mod native_clinic_fixtures;
#[path="native_gift_shop.rs"]
mod native_gift_shop;
#[path="traditional_wood_wall.rs"]
mod traditional_wood_wall;
#[path="native_safari_office.rs"]
mod native_safari_office;
#[path="native_classroom.rs"]
mod native_classroom;
#[path="native_radio_room.rs"]
mod native_radio_room;
#[path="native_radio_studio.rs"]
mod native_radio_studio;
#[path="native_radio_stair.rs"]
mod native_radio_stair;
pub(super) fn radio_stair_support_correction(frame:&VisualWorldFrame,cell:bevy::prelude::UVec2,foot:bevy::prelude::Vec2,resolved:&[f32])->f32 {
    native_radio_stair::support_correction(frame,cell,foot,resolved)
}
#[path="native_house_plant.rs"]
mod native_house_plant;
#[path = "native_tabletop.rs"]
mod native_tabletop;
#[path = "native_table_centerpiece.rs"]
mod native_table_centerpiece;
#[path = "native_home_stair.rs"]
mod native_home_stair;
#[path = "native_mansion_architecture.rs"]
mod native_mansion_architecture;
pub(super) fn built_in_profiles() -> &'static Document {
    static DOCUMENT: OnceLock<Document> = OnceLock::new();
    DOCUMENT.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../../modpacks/voxel-view/profiles.json"
        ))
        .expect("validated source profiles")
    })
}

pub(crate) fn native_healing_owner_allows(map:&str,cells:&[&VisualTile],g:&GridGeometry,
    profiles:&Document,column:usize,row:usize)->bool {
    let canonical=built_in_profiles().objects.iter().find(|o|o.name=="Shared Pokecenter healing machine").unwrap();
    if profiles.objects.iter().any(|o|o.name==canonical.name&&o!=canonical) {return false;}
    if crate::interior_models::selected_model(ModelKind::HealingMachine).is_none() {return false;}
    let custom=Document{objects:profiles.objects.iter()
        .filter(|o|!built_in_profiles().objects.iter().any(|c|c==*o)).cloned().collect(),atmosphere:None};
    if custom.objects.is_empty() {return true;}
    !live::resolve(cells,g.width,g.height,map,Some(&custom)).iter().any(|p|
        p.indices(g.width).any(|i|i%g.width>=column&&i%g.width<column+4&&i/g.width>=row&&i/g.width<row+4))
}

/// Map specific existing source-profile roles onto independently authored art.
/// No prefix/keyword guesses: a new or edited profile does not acquire a model.
fn profile_kind(object: &Object) -> Option<ModelKind> {
    use ModelKind::*;
    let kind = match (object.tileset.as_str(), object.name.as_str()) {
        ("lab", "Lab cabinet 04 0")
        | ("lab", "Lab cabinet 04 2")
        | ("lab", "Lab cabinet 14 0")
        | ("lab", "Lab cabinet 14 2")
        | ("lab", "Shared lab paired bookcases 14") => LabBookcase,
        ("lab", "Lab wall 08") | ("lab", "Lab wall 0d") | ("lab", "Lab wall 17") => WallPanel,
        ("lab", "Lab windows 09") => WindowWall,
        ("lab", "Lab restoration machine") => LabRestorationMachine,
        ("lab", "Shared lab computer workstation") => LabWorkstation,
        ("lab", "Lab central desk assembly") => LabCounter,
        ("house", "New Bark house plant 06") | ("house", "New Bark house plant 07") => PottedPlant,
        ("players_house", "PlayersHouse1F dining table") => DiningTable,
        ("house", "Neighbor dining table") => DiningTable,
        ("players_room", "Player bedroom table") => DiningTable,
        ("pokecenter", "Pokecenter upstairs complete PC")
        | ("pokecenter", "Shared Pokecenter complete PC") => Computer,
        ("pokecenter", "Pokecenter upstairs wall displays 2b")
        | ("pokecenter", "Pokecenter upstairs wall displays 0a") => PictureFrame,
        ("pokecenter", "Pokecenter upstairs seat 2d 0")
        | ("pokecenter", "Pokecenter upstairs seat 2d 2")
        | ("pokecenter", "Pokecenter upstairs seat 29 0")
        | ("pokecenter", "Shared Pokecenter seat 2e")
        | ("pokecenter", "Shared Pokecenter seat 2f") => LinkSeat,
        ("pokecenter", "Shared Pokecenter healing machine") => HealingMachine,
        ("mart", "Shared Mart Ecruteak Mart refrigerator 0")
        | ("mart", "Shared Mart Ecruteak Mart refrigerator 2") => RetailRefrigerator,
        ("mart", "Shared Mart Ecruteak Mart shelf 0")
        | ("mart", "Shared Mart Ecruteak Mart shelf 2")
        | ("mart", "Shared Mart Ecruteak Mart rear shelf 2") => RetailShelf,
        ("facility", "Shared facility paired bookcases 06") => Bookcase,
        ("facility", "Ruins research instrument cabinet") => LabRestorationMachine,
        ("mart", "Shared department store cushion 0")
        | ("mart", "Shared department store cushion 2") => Cushion,
        ("train_station", "Shared station seat 0")
        | ("train_station", "Shared station seat 2")
        | ("train_station", "Shared station column seat 0")
        | ("train_station", "Shared station column seat 2") => Chair,
        ("game_corner", "Shared game corner rear machine 0")
        | ("game_corner", "Shared game corner rear machine 2") => ArcadeMachine,
        ("game_corner", "Shared game row cabinet 7 0")
        | ("game_corner", "Shared game row cabinet 7 2")
        | ("game_corner", "Shared game row cabinet 11 0") => ArcadePair,
        ("game_corner", "Shared game corner stool 5 0")
        | ("game_corner", "Shared game corner stool 5 2")
        | ("game_corner", "Shared game corner stool 6 0")
        | ("game_corner", "Shared game corner stool 6 2") => ArcadeStool,
        ("radio_tower", "Shared radio tower PC") => Computer,
        ("traditional_house", "Traditional house north clock") => PendulumClock,
        ("traditional_house", "Traditional house north cabinet")
        | ("traditional_house", "Traditional house low cabinet") => Cabinet,
        ("traditional_house", "Traditional house plain north wall") => WallPanel,
        ("traditional_house", "Traditional house north wall with radio") => RadioWall,
        ("traditional_house", "Traditional house drawer cabinet") => DrawerCabinet,
        ("traditional_house", "Traditional cushion 10")
        | ("traditional_house", "Traditional cushion 1b")
        | ("traditional_house", "Traditional cushion 19") => Cushion,
        ("gate", "Shared gate north double doors") => GateDoorFrame,
        ("gate", "Shared park gate terminal") => GateTerminal,
        ("gate", "Shared gate wall variant 29")
        | ("gate", "Shared gate wall variant 2e")
        | ("gate", "Shared gate wall variant 2d")
        | ("gate", "Shared park gate terminal adjacent wall")
        | ("gate", "Shared gate counter junction wall")
        | ("gate", "Shared gate counter junction upper wall")
        | ("gate", "Shared gate mirrored counter wall")
        | ("gate", "Shared gate mirrored counter upper wall")
        | ("gate", "Shared gate wall above planters") => WallPanel,
        ("gate", "Shared gate planter 0")
        | ("gate", "Shared gate planter 2")
        | ("gate", "Shared gate corridor planter") => PottedPlant,
        ("radio_tower", "Shared radio tower glass machine") => BroadcastRack,
        ("gate", "Shared gate long planter") => LongPlanter,
        ("facility", "Shared facility potted plant 0d")
        | ("facility", "Shared facility potted plant 0e") => PottedPlant,
        ("lab", "Shared lab round stool") => Stool,
        ("lab", "Shared lab display table") => LabDisplayTable,
        ("house", "Shared house flower stand 30 0")
        | ("house", "Shared house flower stand 30 2")
        | ("house", "Shared house flower stand 31 0")
        | ("house", "Shared house flower stand 31 2") => FlowerStand,
        ("house", "Soul House memorial 28 0")
        | ("house", "Soul House memorial 28 2")
        | ("house", "Soul House memorial 2a 0")
        | ("house", "Soul House memorial 2a 2")
        | ("house", "Soul House memorial 2b 0")
        | ("house", "Soul House memorial 2b 2") => Memorial,
        _ => return None,
    };
    Some(if kind == ModelKind::WallPanel {
        match object.tileset.as_str() {
            "traditional_house" => ModelKind::WallTraditional,
            "lab" | "gate" | "pokecenter" => ModelKind::WallClinical,
            _ => kind,
        }
    } else {
        kind
    })
}

fn cable_club_fixture(object: &Object, map: &str) -> bool {
    super::cable_club::is_map(map) && object.tileset == "pokecenter"
        && matches!(object.name.as_str(), "Pokecenter upstairs complete PC"
            | "Pokecenter upstairs wall displays 2b" | "Pokecenter upstairs wall displays 0a")
}

fn proportions(kind: ModelKind, width: usize, height: usize) -> (f32, f32) {
    use ModelKind::*;
    let source_depth = height as f32 * 8.0;
    let (depth, rise): (f32, f32) = match kind {
        TrashCan => (10.,8.),
        Stool | ArcadeStool => (8.0, 7.0),
        Chair | LinkSeat => (8.0, 11.0),
        Cushion => (15.0, 3.0),
        DiningTable => (source_depth, 10.0),
        LowTable => (source_depth, 6.0),
        CafeTable => (24.0, 16.0),
        BedRed | BedBlue | BedGreen | BedPink | BedFeathery | BedPolkadot | BedPikachu => {
            (source_depth, 11.0)
        }
        PottedPlant | PlantMagna | PlantTropic | PlantJumbo => (12.0, 23.0),
        WallDomestic | WallTraditional | WallClinical | WallAcoustic | ArcadeWall => {
            (3.0, source_depth.max(32.0))
        }
        PictureFrame => (2.0, source_depth),
        ArcadePrizeRack => (10.0, 28.0),
        OlivineStage => (64.0, 8.0),
        CarpetCloth => (source_depth, 0.18),
        StairFlight => (16.0, 22.4),
        OfficePhone => (10.0, 6.0),
        BroadcastRack => (9.0, 30.0),
        TowerReception => (12.0, 18.0),
        LongPlanter => (source_depth, 17.0),
        FlowerStand => (14.0, 16.0),
        Bookcase | LabBookcase => (9.0, 29.0),
        Cabinet | DrawerCabinet => (8.0, 22.0),
        PendulumClock => (7.0, 30.0),
        Computer | LabWorkstation => (12.0, 22.0),
        Television => (9.0, 20.0),
        Radio => (7.0, 12.0),
        Keyboard => (7.0, 1.5),
        GameConsole => (6.0, 3.0),
        Refrigerator | RetailRefrigerator | RetailShelf | GiftShelf | VendingMachine => {
            (10.0, 30.0)
        }
        KitchenStove | KitchenSink => (8.0, 11.0),
        ReceptionCounter | LabCounter => (8.0, 10.0),
        HealingMachine => (12.0, 30.0),
        HealingConsole => (13.0, 13.0),
        ClinicalPartition | TimberPartition => (source_depth, 32.0),
        LabRestorationMachine => (12.0, 22.0),
        LabDisplayTable => (source_depth, 13.0),
        ArcadeMachine => (10.0, 29.0),
        ArcadePair => (14.0, 21.0),
        PrizeBooth => (8.0, 32.0),
        StationBench => (14.0, 14.0),
        StationTurnstile => (18.0, 16.0),
        GateTerminal => (9.0, 21.0),
        BroadcastConsole => (14.0, 19.0),
        WallPanel | WindowWall | RadioWall | GateDoorFrame => (3.0, source_depth.max(32.0)),
        OpenBook => (13.0, 4.0),
        Memorial => (5.0, 14.0),
    };
    // Every axis stays within the source drawing, even for compact variants.
    (
        depth.min(source_depth).max(1.0),
        rise.min(40.0).max(if width == 0 || kind == ModelKind::CarpetCloth { 0.0 } else { 1.0 }),
    )
}
fn reviewed_retail_room(map:&str)->bool {
    matches!(map,"BlackthornMart"|"CherrygroveMart"|"EcruteakMart"|"OlivineMart"|"VioletMart"|"AzaleaMart"|"CeruleanMart"|"FuchsiaMart"|"LavenderMart"|"PewterMart"
        |"SaffronMart"|"VermilionMart"|"ViridianMart"
        |"CeladonDeptStore1F"|"CeladonDeptStore2F"|"CeladonDeptStore3F"
        |"CeladonDeptStore4F"|"CeladonDeptStore5F"|"CeladonDeptStore6F")
        ||map.strip_prefix("GoldenrodDeptStore").is_some_and(|floor|
            matches!(floor,"1F"|"2F"|"3F"|"4F"|"5F"|"6F"))
}

fn mart_display_model(source: &VisualTileSource) -> Option<ModelKind> {
    // These roles follow the complete, tile-verified drawings in mart.rs.
    // Unknown drawings never become merchandise shelves by default.
    crate::mart::display_rack_local(source)?;
    match source.metatile_id {
        0x0b => Some(ModelKind::PottedPlant),
        0x07 | 0x20 => Some(ModelKind::Cushion),
        0x21 => Some(ModelKind::VendingMachine),
        0x12 | 0x13 | 0x2c | 0x2d => Some(ModelKind::RetailShelf),
        _ => None,
    }
}
fn ground_sample(
    cells: &[&VisualTile],
    map: &str,
    tileset: &str,
    tile_index: u16,
) -> Option<usize> {
    cells.iter().position(|tile| {
        tile.source.tileset_id.as_ref() == tileset
            && tile.source.tile_index == tile_index
            && matches!(
                shape_for_source_on_map(map, &tile.source),
                CellShape::Flat | CellShape::Water | CellShape::PlaneAt { height: 0.0 }
            )
    })
}

/// Resolve claims on original cells, before the live-profile or legacy passes.
/// The document is used only to respect customized profile ownership: changed
/// rows or geometry retain the user's live renderer instead of being replaced.
pub(super) fn traditional_tabletop_height(map:&str,cells:&[&VisualTile],g:&GridGeometry,
    profiles:Option<&Document>,column:usize,row:usize)->Option<f32> {
    if !(traditional_room::floor_map(map)||reviewed_native_room(map)||matches!(map,"ElmsLab"|"EarlsPokemonAcademy"))||column>=g.width||row>=g.height
        ||g.width.checked_mul(g.height)!=Some(cells.len()) {return None;}
    // Stationary book/Pokedex props rest on the modeled tabletop. This is
    // presentation support only; the player's walking footing stays unchanged.
    resolve(map,cells,g,profiles).into_iter().find(|p|
        ((native_gift_shop::traditional_table(map,p,cells,g)
            && crate::interior_models::selected_model(ModelKind::LowTable)
                .is_some_and(|m|m.has_native_low_table_materials()))
        || ((native_house_dining_table(map,cells,g,p)||native_classroom::matches(map,p,cells,g)&&p.kind==ModelKind::DiningTable)
            && crate::interior_models::selected_model(ModelKind::DiningTable)
                .is_some_and(|m|m.has_native_dining_table_materials())))
        &&column>=p.column&&column<p.column+p.width&&row>=p.row
        &&row<p.row+if p.kind==ModelKind::DiningTable {native_dining_table_rows(p)}else{p.height})
        .map(|p|(p.base_pixels+p.height_pixels)*g.tile_height/8.)
}

// The apron is part of the complete source claim, beyond the actual tabletop.
fn native_dining_table_rows(p:&Placement)->usize {
    match (p.width,p.height) {(4,5)=>4,(4,3)=>2,_=>p.height}
}

/// A conservative seat candidate test, derived from the same profile kinds
/// and source classifiers as resolve(). It never filters ownership within a
/// candidate scene: the complete resolver still decides overlaps and claims.
pub(super) fn has_seating_source_candidate(map: &str, cells: &[&VisualTile],
    profiles: Option<&Document>) -> bool {
    let seat_kind = |kind| matches!(kind, ModelKind::Stool | ModelKind::ArcadeStool
        | ModelKind::Chair | ModelKind::LinkSeat | ModelKind::StationBench | ModelKind::Cushion);
    let atlases: std::collections::HashSet<&str> = cells.iter()
        .map(|tile|tile.source.tileset_id.as_ref()).collect();
    if built_in_profiles().objects.iter()
        .chain(profiles.into_iter().flat_map(|document|document.objects.iter()))
        .any(|object| profile_kind(object).is_some_and(seat_kind)
            && atlases.contains(object.tileset.as_str())) { return true; }
    // Independent authored seat classifiers are not all profile entries.
    // A single possible source cell is sufficient; exact complete-drawing,
    // map, ground and overlap checks remain in the full resolver below.
    cells.iter().any(|tile| {
        let source=&tile.source;
        crate::house::furniture_local(source).is_some_and(|(_,_,kind)|
            kind==crate::house::FurnitureKind::Stool)
            || crate::house::traditional_cushion_local(source).is_some()
            || crate::cafe::celadon_counter_stool_local(map,source).is_some()
            || crate::pokecenter::link_floor_seat_local(map,source).is_some()
            || crate::pokecom::chair_local(map,source).is_some()
            || crate::train_station::seat_local(map,source).is_some()
            || crate::house::mansion_office_seat_local(map,source).is_some()
            || (native_radio_room::is_map(map)&&native_radio_studio::gold_local(source).is_some())
            || mart_display_model(source).is_some_and(seat_kind)
    })
}

pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    profiles: Option<&Document>,
) -> Vec<Placement> {
    let mut out = Vec::new();
    let mut claimed = vec![false; cells.len()];
    // A user's altered live object retains ownership over all matching cells,
    // including overlaps with older compiled fixture classifiers.
    if let Some(active) = profiles {
        let customized = Document {
            objects: active
                .objects
                .iter()
                .filter(|object| {
                    built_in_profiles()
                        .objects
                        .iter()
                        .any(|canonical| canonical.name == object.name && canonical != *object)
                        || ((super::cable_club::is_map(map)
                            || crate::casino::is_game_corner_map(map)
                            || kanto_native_floor_map(map)
                            || reviewed_retail_room(map)
                            || traditional_room::floor_map(map)
                            || native_bookcase_room(map)
                            || map == "CianwoodGym" || map=="GoldenrodUnderground" || map=="BurnedTower1F"
                            || native_radio_room::is_map(map)
                            || super::modeled_dungeons::tin_tower_guardian_map(map)
                            || super::modeled_dungeons::lighthouse_map(map))
                            && !built_in_profiles().objects.iter().any(|canonical| canonical == *object))
                })
                .cloned()
                .collect(),
            atmosphere: None,
        };
        for placement in live::resolve(
            cells,
            geometry.width,
            geometry.height,
            map,
            Some(&customized),
        ) {
            for index in placement.indices(geometry.width) {
                claimed[index] = true;
            }
        }
    }
    native_clinic_fixtures::candidates(map,cells,geometry,&mut claimed,&mut out);
    native_safari_office::candidates(map,cells,geometry,&mut claimed,&mut out);
    native_classroom::candidates(map,cells,geometry,&mut claimed,&mut out);
    native_radio_room::candidates(map,cells,geometry,&mut claimed,&mut out);
    native_radio_studio::candidates(map,cells,geometry,&mut claimed,&mut out);
    native_radio_stair::candidates(map,cells,geometry,&mut claimed,&mut out);
    append_prize_booth_candidate(map, cells, geometry, &mut claimed, &mut out);
    for object in &built_in_profiles().objects {
        let Some(kind) = profile_kind(object) else {
            continue;
        };
        // This room's solid plank course uses its own complete native drawing.
        if map=="WiseTriosRoom"&&object.tileset=="traditional_house"
            &&object.metatile==3&&kind==ModelKind::WallTraditional {continue;}
        let kind=if kind==ModelKind::FlowerStand && reviewed_native_room(map) && object.tileset=="house"
            && matches!(object.metatile,0x30|0x31) {ModelKind::LowTable}else{kind};
        // Elm's balls are authoritative objects, not fixed display specimens.
        // Oak's matching desk is empty. Both need the bare source table.
        let kind=if matches!(map,"OaksLab"|"ElmsLab") && object.name=="Shared lab display table" {
            ModelKind::DiningTable
        } else {kind};
        let cable_fixture = cable_club_fixture(object, map);
        if !cable_fixture && (object.map.as_deref().is_some_and(|id| id != map)
            || object
                .maps
                .as_ref()
                .is_some_and(|ids| !ids.iter().any(|id| id == map)))
        {
            continue;
        }
        if let Some(active) =
            profiles.and_then(|d| d.objects.iter().find(|o| o.name == object.name))
        {
            if active != object {
                continue;
            }
        }
        let ground_tile = if object.tileset == "mart"
            && matches!(kind, ModelKind::RetailRefrigerator | ModelKind::RetailShelf)
        {
            crate::mart::ground_tile_for_map(map)
        } else {
            object.ground
        };
        let Some(ground) = ground_sample(cells, map, &object.tileset, ground_tile) else {
            continue;
        };
        let (w, h) = (object.tiles[0].len(), object.tiles.len());
        if w > geometry.width || h > geometry.height {
            continue;
        }
        for row in 0..=geometry.height - h {
            for column in 0..=geometry.width - w {
                let complete = (0..h).all(|y| {
                    (0..w).all(|x| {
                        let index = (row + y) * geometry.width + column + x;
                        let s = &cells[index].source;
                        !claimed[index]
                            && s.tileset_id.as_ref() == object.tileset
                            && s.metatile_id
                                == object.metatiles.as_ref().map_or(object.metatile, |blocks| {
                                    blocks[(usize::from(object.origin[1]) + y) / 4]
                                        [(usize::from(object.origin[0]) + x) / 4]
                                })
                            && usize::from(s.subtile_column)
                                == (usize::from(object.origin[0]) + x) % 4
                            && usize::from(s.subtile_row) == (usize::from(object.origin[1]) + y) % 4
                            && s.tile_index == object.tiles[y][x]
                    })
                });
                if !complete {
                    continue;
                }
                let (depth_pixels, height_pixels) = proportions(kind, w, h);
                let (depth_pixels,height_pixels)=if kind==ModelKind::Cushion && object.tileset=="mart" {
                    (12.0,7.0)
                } else if map=="PlayersHouse2F" && kind==ModelKind::DiningTable
                    && object.tileset=="players_room" && object.name=="Player bedroom table" && w==4 && h==3 {
                    (16.,10.)
                } else {(depth_pixels,height_pixels)};
                let mut p = Placement {
                    kind,
                    column,
                    row,
                    // Each paired profile is a sign next to a real DOOR.
                    // Keep the full source guard, but give PictureFrame only
                    // the sign half. cable_club supplies the open door frame.
                    width: if cable_fixture && kind == ModelKind::PictureFrame { 2 } else { w },
                    height: h,
                    ground,
                    depth_pixels,
                    height_pixels,
                    base_pixels: 0.0,
                    footing_pixels: object.footing_pixels,
                };
                if native_elm_fixture_source(map,&p,cells,geometry).is_some() {
                    p.depth_pixels=24.;p.height_pixels=8.;
                }
                if reviewed_arcade_room(map) && matches!(kind,ModelKind::ArcadeMachine|ModelKind::ArcadePair)
                    && native_arcade_source(map,&p,cells,geometry).is_none() {continue;}
                if kind==ModelKind::LinkSeat && kanto_native_floor_map(map) && map.ends_with("Pokecenter2FBeta")
                    && native_link_seat_source(map,&p,cells,geometry).is_none() {continue;}
                if kind==ModelKind::LowTable && object.tileset=="house" && matches!(object.metatile,0x30|0x31)
                    && reviewed_native_room(map) && !native_small_display_source(map,&p,cells,geometry) {continue;}
                for i in p.indices(geometry.width) {
                    claimed[i] = true;
                }
                out.push(p);
            }
        }
    }
    // Older grouped matchers encode exact block phase and source tile identity.
    // These objects are independent from the live catalog and take only free cells.
    let mut add = |groups: Vec<TreePlacement>,
                   kind_for: &dyn Fn(&VisualTileSource) -> ModelKind| {
        for group in groups {
            let source = &cells[group.row * geometry.width + group.column].source;
            let kind = kind_for(source);
            // A damaged full rank must not silently become partial machines.
            if reviewed_arcade_room(map) && kind==ModelKind::ArcadeMachine
                && matches!(source.metatile_id,7|11) {continue;}
            let Some(ground) =
                ground_sample(cells, map, &source.tileset_id, if matches!(map,"MountMoonGiftShop"|"MahoganyMart1F")&&kind==ModelKind::GiftShelf {1}else{group.ground_tile_index})
            else {
                continue;
            };
            let (depth_pixels, height_pixels) = if kind == ModelKind::Chair
                && crate::pokecom::chair_local(map,source).is_some() {
                (8.,7.)
            } else if kind == ModelKind::Cushion
                && source.tileset_id.as_ref() == "mart" {
                (12.0,7.0)
            } else if kind==ModelKind::DrawerCabinet && reviewed_native_room(map)
                && source.tileset_id.as_ref()=="players_house" && source.metatile_id==0x11 {
                (8.0,16.0)
            } else if kind==ModelKind::Radio && reviewed_native_room(map)
                && source.tileset_id.as_ref()=="house" && source.metatile_id==0x1d {
                (8.,16.)
            } else if map=="RedsHouse2F" && kind==ModelKind::DiningTable
                && source.tileset_id.as_ref()=="players_house" && source.metatile_id==0x20 {
                (16.0,10.0)
            } else if source.tileset_id.as_ref()=="train_station" && kind==ModelKind::WallClinical
                && group.width==4 && group.height==8 {
                (3.,32.)
            } else if source.tileset_id.as_ref()=="train_station" && kind==ModelKind::WallPanel
                && matches!(group.width,2|4) && group.height==2 {
                (1.,11.)
            } else if source.tileset_id.as_ref()=="train_station" && kind==ModelKind::VendingMachine {
                (group.height as f32*8.,20.)
            } else if source.tileset_id.as_ref()=="train_station" && kind==ModelKind::LongPlanter {
                (group.height as f32*6.,17.)
            } else { proportions(kind, group.width, group.height) };
            let p = Placement {
                kind,
                column: group.column,
                row: group.row,
                width: group.width,
                height: group.height,
                ground,
                depth_pixels,
                height_pixels,
                base_pixels: 0.0,
                footing_pixels: None,
            };
            if source.tileset_id.as_ref()=="train_station"
                && ((kind==ModelKind::WallClinical && !native_station_wall_source(map,&p,cells,geometry))
                    || (kind==ModelKind::LongPlanter && !native_station_plant_source(map,&p,cells,geometry))
                    || (kind==ModelKind::VendingMachine && !native_station_ticket_source(map,&p,cells,geometry))) {continue;}
            if reviewed_arcade_room(map) && matches!(kind,ModelKind::ArcadeMachine|ModelKind::ArcadePair)
                && native_arcade_source(map,&p,cells,geometry).is_none() {continue;}
            if kind==ModelKind::LinkSeat && kanto_native_floor_map(map) && map.ends_with("Pokecenter2FBeta")
                && native_link_seat_source(map,&p,cells,geometry).is_none() {continue;}
            if matches!(map,"CeladonMansion2F"|"CeladonMansion3F")
                && ((kind==ModelKind::LabWorkstation && native_mansion_workstation_source(map,&p,cells,geometry).is_none())
                    || (kind==ModelKind::LinkSeat && native_link_seat_source(map,&p,cells,geometry).is_none())) {continue;}
            if p.indices(geometry.width).any(|i| claimed[i]) {
                continue;
            }
            for i in p.indices(geometry.width) {
                claimed[i] = true;
            }
            out.push(p);
        }
    };
    add(house_plant_placements(cells, geometry), &|_| {
        ModelKind::PottedPlant
    });
    add(house_bookcase_placements(cells, geometry), &|_| {
        ModelKind::Bookcase
    });
    add(house_upright_fixture_placements(cells, geometry), &|s| {
        if s.metatile_id == 0x1e {
            ModelKind::Television
        } else {
            ModelKind::Radio
        }
    });
    add(players_house_bookcase_placements(cells, geometry), &|_| {
        ModelKind::Bookcase
    });
    // Player-home fixture material uses its original north-course floor sample.
    let mut player_fixtures = players_house_upright_fixture_placements(cells, geometry);
    for p in &mut player_fixtures {
        p.ground_tile_index = crate::players_house::FLOOR_TILE;
    }
    add(
        player_fixtures,
        &|s| match (s.metatile_id, s.subtile_column) {
            (0x07, 0 | 1) => ModelKind::KitchenStove,
            (0x07, _) => ModelKind::KitchenSink,
            (0x0f, _) => ModelKind::Refrigerator,
            (0x11, 0 | 1) => ModelKind::Television,
            _ if reviewed_native_room(map) => ModelKind::DrawerCabinet,
            _ => ModelKind::Cabinet,
        },
    );
    add(players_house_tv_placements(cells, geometry), &|_| {
        ModelKind::Television
    });
    add(players_house_console_placements(cells, geometry), &|_| {
        ModelKind::GameConsole
    });
    add(players_house_bed_placements(cells, geometry), &|_| {
        ModelKind::BedRed
    });
    add(
        player_bed_placements(cells, geometry),
        &|s| match s.metatile_id {
            0x1c => ModelKind::BedPink,
            0x1d => ModelKind::BedPolkadot,
            0x1e => ModelKind::BedPikachu,
            _ => ModelKind::BedFeathery,
        },
    );
    add(
        traditional_gift_shop_shelf_placements(map, cells, geometry),
        &|_| ModelKind::GiftShelf,
    );
    add(
        traditional_house_cushion_placements(map, cells, geometry),
        &|_| ModelKind::Cushion,
    );
    add(soul_house_bench_placements(cells, geometry), &|_| {
        ModelKind::Memorial
    });
    add(house_open_book_placements(cells, geometry), &|_| {
        ModelKind::OpenBook
    });
    add(house_display_table_placements(cells, geometry), &|_| {
        if reviewed_native_room(map) {ModelKind::LowTable}else{ModelKind::FlowerStand}
    });
    add(grouped_flat_card_placements(cells,geometry,0x01,false,|s|
        crate::house::large_table_local(map,s)),&|_|ModelKind::DiningTable);
    add(grouped_flat_card_placements(cells,geometry,0x11,false,|s|
        crate::house::mansion_lounge_table_local(map,s)),&|_|ModelKind::DiningTable);
    add(grouped_flat_card_placements(cells,geometry,0x11,false,|s|
        crate::house::mansion_workstation_local(map,s)),&|_|ModelKind::LabWorkstation);
    if map=="CeladonCafe" {
        add(grouped_flat_card_placements(cells,geometry,0x01,false,|s| {
            crate::cafe::celadon_table_local(map,s).map(|(x,y)|(x,y,4,5))
        }),&|_|ModelKind::CafeTable);
        add(grouped_flat_card_placements(cells,geometry,0x01,false,|s| {
            crate::cafe::celadon_counter_stool_local(map,s).map(|(x,y)|(x,y,2,2))
        }),&|_|ModelKind::ArcadeStool);
    }
    add(
        mr_pokemon_work_counter_placements(map, cells, geometry),
        &|_| ModelKind::LabCounter,
    );
    let displays = mart_display_rack_placements(map, cells, geometry)
        .into_iter().filter(|p| {
            mart_display_model(&cells[p.row * geometry.width + p.column].source).is_some()
        }).collect();
    add(displays, &|s| mart_display_model(s).expect("explicitly classified Mart drawing"));
    add(
        pokecenter_healing_console_placements(map, cells, geometry),
        &|_| ModelKind::HealingConsole,
    );
    add(
        pokecenter_link_floor_seat_placements(map, cells, geometry),
        &|_| ModelKind::LinkSeat,
    );
    add(
        pokecom_workstation_placements(map, cells, geometry),
        &|_| ModelKind::Computer,
    );
    add(pokecom_plant_placements(map, cells, geometry), &|_| {
        ModelKind::PottedPlant
    });
    add(pokecom_chair_placements(map, cells, geometry), &|_| {
        ModelKind::Chair
    });
    add(train_station_seat_placements(map, cells, geometry), &|_| {
        ModelKind::Chair
    });
    add(grouped_flat_card_placements(cells,geometry,crate::train_station::FLOOR_TILE,true,|s| {
        crate::train_station::wall_local(map,s)
    }),&|_|ModelKind::WallClinical);
    add(grouped_flat_card_placements(cells,geometry,0x3e,false,|s| {
        crate::train_station::railing_local(map,s)
    }),&|_|ModelKind::WallPanel);
    add(grouped_flat_card_placements(cells,geometry,crate::train_station::FLOOR_TILE,true,|s| {
        crate::train_station::planter_drawing_local(map,s)
    }),&|_|ModelKind::LongPlanter);
    add(grouped_flat_card_placements(cells,geometry,0x01,false,|s| {
        mirrored_gate_planter_local(map,s)
    }),&|_|ModelKind::LongPlanter);
    add(grouped_flat_card_placements(cells,geometry,crate::train_station::FLOOR_TILE,true,|s| {
        crate::train_station::ticket_box_local(map,s)
    }),&|_|ModelKind::VendingMachine);
    add(train_station_gate_placements(map, cells, geometry), &|_| {
        ModelKind::StationTurnstile
    });
    if crate::casino::is_game_corner_map(map) {
        add(casino_terminal_placements(cells, geometry), &|_| {
            ModelKind::ArcadeMachine
        });
        add(casino_slot_machine_placements(cells, geometry), &|_| {
            ModelKind::ArcadeMachine
        });
        add(casino_plant_placements(cells, geometry), &|_| {
            ModelKind::PottedPlant
        });
    }
    // The player's computer is one screen-plus-keyboard assembly; the separate
    // source rows are never interpreted as two full computer towers.
    add(
        grouped_flat_card_placements(cells, geometry, 0x02, false, |s| {
            crate::interior::player_room_pc_monitor_local(s)
                .map(|(x, y)| (x, y, 2, 3))
                .or_else(|| {
                    crate::interior::player_room_pc_keyboard_local(s).map(|(x, _)| (x, 2, 2, 3))
                })
        }),
        &|_| ModelKind::Computer,
    );
    add(
        grouped_flat_card_placements(cells, geometry, 0x01, false, |s| {
            crate::house::furniture_local(s).and_then(|(x, y, k)| {
                (k == crate::house::FurnitureKind::Stool).then_some((x, y, 2, 2))
            })
        }),
        &|_| ModelKind::Stool,
    );
    if map=="RedsHouse2F" {
        add(grouped_flat_card_placements(cells,geometry,0x01,false,|s| {
            crate::players_house::red_desk_local(s).map(|(x,y)|(x,y,4,3))
        }),&|_|ModelKind::DiningTable);
    }
    add(wise_trios_divider_placements(map,cells,geometry),&|_|ModelKind::WallTraditional);
    drop(add);
    // Complete dining drawings that cross metatile boundaries are already
    // validated by the established table matcher (including native corners).
    for table in house_table_placements(cells, geometry) {
        let s = &cells[table.row * geometry.width + table.column].source;
        let Some(ground) = ground_sample(cells, map, &s.tileset_id, table.ground_tile_index) else {
            continue;
        };
        let native_player=reviewed_native_room(map) && s.tileset_id.as_ref()=="players_house";
        if native_player && !native_player_dining_table_complete(cells,geometry,table.column,table.row) {continue;}
        let p = Placement {
            kind: if s.tileset_id.as_ref() == "traditional_house" {
                ModelKind::LowTable
            } else {
                ModelKind::DiningTable
            },
            column: table.column,
            row: table.row,
            width: 4,
            height: if native_player {5}else{4},
            ground,
            depth_pixels: 32.0,
            // The old source extrusion height is not the imported model's
            // scale. Use the same dining proportions as profile-resolved tables.
            height_pixels: if s.tileset_id.as_ref()=="traditional_house" {
                table.height_pixels
            } else {proportions(ModelKind::DiningTable,4,4).1},
            base_pixels: 0.0,
            footing_pixels: None,
        };
        if p.indices(geometry.width).any(|i| claimed[i]) {
            continue;
        }
        for i in p.indices(geometry.width) {
            claimed[i] = true;
        }
        out.push(p);
    }
    native_gift_shop::append_counter(map,cells,geometry,&mut claimed,&mut out);
    append_counter_candidates(map, cells, geometry, &mut claimed, &mut out);
    append_casino_candidates(map, cells, geometry, &mut claimed, &mut out);
    append_cafe_architecture_candidates(map,cells,geometry,&mut claimed,&mut out);
    append_cianwood_walls(map, cells, geometry, &mut claimed, &mut out);
    native_mansion_architecture::candidates(map,cells,geometry,&mut claimed,&mut out);
    append_signature_candidates(map, cells, geometry, &mut claimed, &mut out);
    native_clinic::append_mirrored_wall(map,cells,geometry,&mut claimed,&mut out);
    native_clinic::append_pc_headers(map,cells,geometry,&mut claimed,&mut out);
    append_stair_candidates(map, cells, geometry, &mut claimed, &mut out);
    append_player_house_wall_fragments(map, cells, geometry, &mut claimed, &mut out);
    if let Some(ground)=ground_sample(cells,map,"mansion",0x11) {
        for p in &mut out {
            if native_mansion_corridor_furniture(map,p,cells,geometry) {
                // These complete furnishings stand in the pink room inset,
                // including block0f's cabinet/clock beside the gold hallway.
                p.ground=ground;
            }
        }
    }
    for p in &mut out {
        if let Some((depth,height))=native_radio_studio::equipment_size(map,p,cells,geometry) {
            p.depth_pixels=depth;p.height_pixels=height;
        }
    }
    out
}

// These are the remaining gold wall courses beside the PC and stairs. Match
// the full drawing and phase, then reuse the same closed native wall geometry.
fn append_player_house_wall_fragments(map: &str, cells: &[&VisualTile],
    g: &GridGeometry, claimed: &mut [bool], out: &mut Vec<Placement>) {
    if !reviewed_native_room(map) { return; }
    let Some(ground)=ground_sample(cells,map,"players_house",crate::players_house::FLOOR_TILE) else {return;};
    for (block,ox,oy,w,h) in [(0x0b,0,0,2,2),(0x10,0,0,4,1),(0x10,2,1,2,1)] {
        if w>g.width || h>g.height {continue;}
        for row in 0..=g.height-h {for column in 0..=g.width-w {
            if row<oy {continue;}
            let complete=(0..h).all(|y|(0..w).all(|x| {
                let i=(row+y)*g.width+column+x;let s=&cells[i].source;
                !claimed[i] && s.tileset_id.as_ref()=="players_house"
                    && s.metatile_id==block && s.tile_index==0x11
                    && s.subtile_column as usize==ox+x && s.subtile_row as usize==oy+y
            }));
            if !complete {continue;}
            let p=Placement{kind:ModelKind::WallDomestic,column,row,width:w,height:h,
                ground,depth_pixels:3.,height_pixels:h as f32*8.,
                base_pixels:(2-oy-h) as f32*8.,footing_pixels:None};
            for i in p.indices(g.width) {claimed[i]=true;}
            out.push(p);
        }}
    }
}

fn append_counter_candidates(
    map: &str,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    // Exact source strips that already have native top+front semantics. End
    // pieces stay separate so L-shaped access and service openings stay open.
    for (index, tile) in cells.iter().enumerate() {
        let s = &tile.source;
        let (ox, oy, w, ground, kind) = match (s.tileset_id.as_ref(), s.metatile_id) {
            ("pokecenter", 0x07) if kanto_native_floor_map(map) && s.subtile_column==0 && s.subtile_row==0 => {
                (0,0,2,0x11,ModelKind::ReceptionCounter)
            }
            ("pokecenter", 0x05 | 0x06 | 0x07) if s.subtile_column == 0 && s.subtile_row == 0 => {
                (0, 0, 4, 0x11, ModelKind::ReceptionCounter)
            }
            ("pokecenter", 0x38) if s.subtile_column == 0 && s.subtile_row == 0 => {
                (0, 0, 2, 0x11, ModelKind::ReceptionCounter)
            }
            ("mart", 0x22) if s.subtile_column == 0 && s.subtile_row == 0 => (
                0,
                0,
                2,
                crate::mart::ground_tile_for_map(map),
                ModelKind::ReceptionCounter,
            ),
            ("mart", 0x0c) if s.subtile_column == 2 && s.subtile_row == 0 => (
                2,
                0,
                2,
                crate::mart::ground_tile_for_map(map),
                ModelKind::ReceptionCounter,
            ),
            ("mart", 0x0d) if s.subtile_column == 0 && s.subtile_row == 0 => (
                0,
                0,
                4,
                crate::mart::ground_tile_for_map(map),
                ModelKind::ReceptionCounter,
            ),
            ("mart", 0x0e) if s.subtile_column == 0 && s.subtile_row == 0 => (
                0,
                0,
                2,
                crate::mart::ground_tile_for_map(map),
                ModelKind::ReceptionCounter,
            ),
            _ => continue,
        };
        let (column, row) = (index % geometry.width, index / geometry.width);
        if column + w > geometry.width || row + 2 > geometry.height {
            continue;
        }
        let Some(ground) = ground_sample(cells, map, &s.tileset_id, ground) else {
            continue;
        };
        let complete = (0..2).all(|y| {
            (0..w).all(|x| {
                let i = (row + y) * geometry.width + column + x;
                let t = &cells[i].source;
                if claimed[i]
                    || t.tileset_id != s.tileset_id
                    || t.metatile_id != s.metatile_id
                    || t.subtile_column as usize != ox + x
                    || t.subtile_row as usize != oy + y
                {
                    return false;
                }
                let shape = if t.tileset_id.as_ref() == "pokecenter" {
                    crate::pokecenter::shape(map, t)
                } else {
                    crate::mart::shape(t)
                };
                match (y, shape) {
                    (
                        0,
                        Some(CellShape::RaisedTop {
                            height: 8.0,
                            solid: SolidKind::Prop,
                        }),
                    ) => true,
                    (
                        1,
                        Some(CellShape::LedgeBand {
                            face: LedgeFace::South,
                            band_count: 1,
                            height: 8.0,
                            ..
                        }),
                    ) => true,
                    _ => false,
                }
            })
        });
        if !complete {
            continue;
        }
        let p = Placement {
            kind,
            column,
            row,
            width: w,
            height: 2,
            ground,
            depth_pixels: 8.0,
            height_pixels: 8.0,
            base_pixels: 0.0,
            footing_pixels: None,
        };
        if s.tileset_id.as_ref()=="mart" && (kanto_native_floor_map(map)||store_floor_map(map))
            && native_mart_counter_source(map,&p,cells,geometry).is_none() {continue;}
        if s.tileset_id.as_ref()=="pokecenter" && kanto_native_floor_map(map)
            && native_center_counter_source(map,&p,cells,geometry).is_none() {continue;}
        for i in p.indices(geometry.width) {
            claimed[i] = true;
        }
        out.push(p);
    }
}

fn append_cafe_architecture_candidates(map:&str,cells:&[&VisualTile],g:&GridGeometry,
    claimed:&mut[bool],out:&mut Vec<Placement>) {
    if map!="CeladonCafe" {return;}
    let Some(ground)=ground_sample(cells,map,"game_corner",1) else{return;};
    for (i,tile) in cells.iter().enumerate() {
        let source=&tile.source;
        if source.tileset_id.as_ref()!="game_corner" || source.subtile_column!=0 || source.subtile_row!=0 {continue;}
        let (column,row)=(i%g.width,i/g.width);
        let mut piece=|kind,rect:[usize;4],expected:&[u16]| {
            let [x,y,w,h]=rect;if column+x+w>g.width||row+y+h>g.height {return;}
            if !(0..h).all(|dy|(0..w).all(|dx| {
                let index=(row+y+dy)*g.width+column+x+dx;let s=&cells[index].source;
                !claimed[index]&&s.tileset_id.as_ref()=="game_corner"&&s.metatile_id==source.metatile_id
                    &&s.subtile_column as usize==x+dx&&s.subtile_row as usize==y+dy
                    &&s.tile_index==expected[dy*w+dx]
            })) {return;}
            let wall=kind==ModelKind::ArcadeWall;
            let p=Placement{kind,column:column+x,row:row+y,width:w,height:h,ground,
                depth_pixels:if wall {3.}else{10.},height_pixels:if wall {h as f32*8.}else if kind==ModelKind::TrashCan {8.}else{28.},
                base_pixels:if wall {(2-y-h)as f32*8.}else{0.},footing_pixels:None};
            for index in p.indices(g.width) {claimed[index]=true;}out.push(p);
        };
        match source.metatile_id {
            0x04=>piece(ModelKind::ArcadePrizeRack,[0,0,4,3],&casino_drawing(4).unwrap()[..12]),
            0x29=>{piece(ModelKind::ArcadeWall,[0,0,4,2],&[2;8]);
                piece(ModelKind::TrashCan,[2,2,2,2],&[0x46,0x47,0x56,0x57]);},
            0x02|0x2d=>piece(ModelKind::ArcadeWall,[0,0,4,2],&[2;8]),
            0x13=>{piece(ModelKind::ArcadeWall,[0,0,2,1],&[2;2]);
                piece(ModelKind::ArcadeWall,[2,0,2,2],&[2;4]);},
            0x28=>piece(ModelKind::ArcadeWall,[0,0,4,2],&[2,2,6,7,2,2,8,9]),
            _=>{},
        }
    }
}

fn native_player_dining_table_complete(cells:&[&VisualTile],g:&GridGeometry,column:usize,row:usize)->bool {
    let object=built_in_profiles().objects.iter().find(|o|o.name=="PlayersHouse1F dining table").unwrap();
    let h=object.tiles.len();let w=object.tiles[0].len();
    if column+w>g.width || row+h>g.height {return false;}
    (0..h).all(|y|(0..w).all(|x| {
        let s=&cells[(row+y)*g.width+column+x].source;
        let ox=object.origin[0]as usize+x;let oy=object.origin[1]as usize+y;
        s.tileset_id.as_ref()=="players_house" && s.subtile_column as usize==ox%4
            && s.subtile_row as usize==oy%4 && s.tile_index==object.tiles[y][x]
            && s.metatile_id==object.metatiles.as_ref().unwrap()[oy/4][ox/4]
    }))
}

fn native_house_window_source(p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    p.kind==ModelKind::WindowWall && p.width==2 && p.height==2
        && INTERIOR_DRAWINGS.iter().any(|d|d.kind==ModelKind::WindowWall
            && matches!(d.tileset,"house"|"players_house") && d.size==[p.width,p.height]
            && p.indices(g.width).enumerate().all(|(j,i)| {
                let s=&cells[i].source;
                s.tileset_id.as_ref()==d.tileset && s.metatile_id==d.block
                    && s.subtile_column==d.origin[0]+(j%p.width)as u8
                    && s.subtile_row==d.origin[1]+(j/p.width)as u8 && s.tile_index==d.tiles[j]
            }))
}

fn native_house_television_source(p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<[[usize;2];5]> {
    if p.kind!=ModelKind::Television || p.width!=2 || !(2..=3).contains(&p.height) {return None;}
    let source=&cells[p.row*g.width+p.column].source;
    if p.height==2 {
        return (p.indices(g.width).enumerate().all(|(j,i)|
            crate::players_house::tv_local(&cells[i].source)==Some(((j%2)as u8,(j/2)as u8))))
            .then_some([[3,2],[3,3],[1,3],[3,11],[3,2]]);
    }
    let timber=match source.tileset_id.as_ref() {
        "house" if source.metatile_id==0x1e && p.indices(g.width).enumerate().all(|(j,i)|
            crate::house::upright_fixture_local(&cells[i].source)==Some(((j%2)as u8,(j/2)as u8,2,3)))=>[3,23],
        "players_house" if source.metatile_id==0x11 && p.indices(g.width).enumerate().all(|(j,i)|
            crate::players_house::upright_fixture_local(&cells[i].source)==Some(((j%2)as u8,(j/2)as u8,2,3)))=>[1,16],
        "players_house" if source.metatile_id==2 && p.indices(g.width).enumerate().all(|(j,i)| {
            let s=&cells[i].source;
            s.tileset_id.as_ref()=="players_house" && s.metatile_id==2
                && s.subtile_column==2+(j%2)as u8 && s.subtile_row==1+(j/2)as u8
                && s.tile_index==[6,7,0x16,0x17,0x18,0x19][j]
        })=>[2,18],
        "players_room" if source.metatile_id==3 && p.indices(g.width).enumerate().all(|(j,i)| {
            let s=&cells[i].source;
            s.tileset_id.as_ref()=="players_room" && s.metatile_id==3
                && s.subtile_column as usize==j%2 && s.subtile_row as usize==1+j/2
                && s.tile_index==[0x3b,0x3c,0x4b,0x4c,0x5b,0x5c][j]
        })=>[3,19],
        _=>return None,
    };
    Some([timber,[3,3],[1,3],[3,11],[3,2]])
}

fn native_player_bedroom_table(map:&str,cells:&[&VisualTile],g:&GridGeometry,p:&Placement)->bool {
    map=="PlayersHouse2F" && p.kind==ModelKind::DiningTable && p.width==4 && p.height==3
        && p.indices(g.width).enumerate().all(|(j,i)| {
            let source=&cells[i].source;
            source.tileset_id.as_ref()=="players_room" && source.metatile_id==7
                && source.subtile_column as usize==j%4 && source.subtile_row as usize==j/4
                && source.tile_index==[16,17,17,18,32,33,33,34,48,49,49,50][j]
        })
}

fn native_house_dining_table(map:&str,cells:&[&VisualTile],g:&GridGeometry,p:&Placement)->bool {
    native_player_bedroom_table(map,cells,g,p) || native_mansion_lounge_table(map,p,cells,g) || native_oak_table(map,p,cells,g) || native_large_house_table(map,p,cells,g) || reviewed_native_room(map) && p.kind==ModelKind::DiningTable && p.width==4
        && ((p.height==4 && p.indices(g.width).all(|i|cells[i].source.tileset_id.as_ref()=="house")
            && house_table_placements(cells,g).iter().any(|t|t.column==p.column && t.row==p.row))
            || (p.height==5 && native_player_dining_table_complete(cells,g,p.column,p.row))
            || (map=="RedsHouse2F" && p.height==3 && p.indices(g.width).enumerate().all(|(j,i)|
                crate::players_house::red_desk_local(&cells[i].source)==Some(((j%4)as u8,(j/4)as u8)))))
}
fn native_mansion_lounge_table(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    p.kind==ModelKind::DiningTable && p.width==4 && p.height==4
        && p.indices(g.width).enumerate().all(|(j,i)|crate::house::mansion_lounge_table_local(map,&cells[i].source)
            ==Some(((j%4)as u8,(j/4)as u8,4,4)))
}
fn native_oak_table(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    matches!(map,"OaksLab"|"ElmsLab") && p.kind==ModelKind::DiningTable && p.width==6 && p.height==3
        && p.indices(g.width).enumerate().all(|(j,i)| {
            let x=j%6;let y=j/6;let s=&cells[i].source;
            s.tileset_id.as_ref()=="lab"
                && s.metatile_id==[[6,7],[10,11]][(y+2)/4][x/4]
                && s.subtile_column as usize==x%4 && s.subtile_row as usize==(y+2)%4
                && s.tile_index==[[5,6,6,6,6,7],[21,22,22,22,22,23],[37,38,38,38,38,39]][y][x]
        })
}
fn native_large_house_table(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    p.kind==ModelKind::DiningTable && p.width==8 && p.height==8
        && p.indices(g.width).enumerate().all(|(j,i)|crate::house::large_table_local(map,&cells[i].source)
            ==Some(((j%8)as u8,(j/8)as u8,8,8)))
}
fn native_small_display_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if !reviewed_native_room(map) || p.kind!=ModelKind::LowTable || p.width!=2 || p.height!=2 {return false;}
    let block=cells[p.row*g.width+p.column].source.metatile_id;
    p.indices(g.width).enumerate().all(|(j,i)|cells[i].source.metatile_id==block
        && crate::house::display_table_local(&cells[i].source)==Some(((j%2)as u8,(j/2)as u8)))
}

fn native_station_plant_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    let block=cells[p.row*g.width+p.column].source.metatile_id;
    p.kind==ModelKind::LongPlanter && p.width==2 && p.indices(g.width).enumerate().all(|(j,i)|
        cells[i].source.metatile_id==block && crate::train_station::planter_drawing_local(map,&cells[i].source)
            ==Some(((j%2)as u8,(j/2)as u8,2,p.height)))
}

fn native_port_plant(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if map!="VermilionPortPassage"||p.kind!=ModelKind::PottedPlant||p.width!=2||p.height!=3 {return false;}
    let first=&cells[p.row*g.width+p.column].source;
    if !matches!((first.metatile_id,first.subtile_column,first.subtile_row),(41,2,0)|(42,0,0)) {return false;}
    p.indices(g.width).enumerate().all(|(j,i)| {let s=&cells[i].source;
        s.tileset_id.as_ref()=="underground"&&s.metatile_id==first.metatile_id
            &&s.subtile_column==first.subtile_column+(j%2)as u8&&s.subtile_row==(j/2)as u8
            &&s.tile_index==[30,31,46,47,62,63][j]})
}

fn mirrored_gate_planter_local(map:&str,s:&VisualTileSource)->Option<(u8,u8,usize,usize)> {
    if map!="GoldenrodUnderground"||s.tileset_id.as_ref()!="gate"||s.metatile_id!=0x3a
        ||s.subtile_column<2||s.subtile_column>3||s.subtile_row>3 {return None;}
    let x=s.subtile_column-2;let y=s.subtile_row;
    let tile=match y {0=>[0x38,0x39][x as usize],3=>[0x15,0x16][x as usize],_=>[0x05,0x06][x as usize]};
    (s.tile_index==tile).then_some((x,y,2,4))
}

fn native_gate_plant_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if !(super::gate_counters::reviewed_gate_map(map)||map=="GoldenrodUnderground")
        ||p.base_pixels!=0.||p.width!=2||cells.len()!=g.width*g.height
        ||p.column+2>g.width||p.row+p.height>g.height {return false;}
    let first=&cells[p.row*g.width+p.column].source;
    let long=p.kind==ModelKind::LongPlanter&&p.height==4&&map=="GoldenrodUnderground"
        &&matches!((first.metatile_id,first.subtile_column,first.subtile_row),(0x38,0,0)|(0x3a,2,0));
    let small=p.kind==ModelKind::PottedPlant&&p.height==2&&match first.metatile_id {
        0x39=>matches!((first.subtile_column,first.subtile_row),(0,2)|(2,2)),
        0x33=>map=="GoldenrodUnderground"&&(first.subtile_column,first.subtile_row)==(2,2),
        _=>false,
    };
    if !small&&!long{return false;}
    p.indices(g.width).enumerate().all(|(j,i)| {
        let source=&cells[i].source;let row=j/2;
        source.tileset_id.as_ref()=="gate"&&source.metatile_id==first.metatile_id
            &&source.subtile_column==first.subtile_column+(j%2)as u8
            &&usize::from(source.subtile_row)==usize::from(first.subtile_row)+row
            &&source.tile_index==if row==0 {[0x38,0x39][j%2]}
                else if row==p.height-1 {[0x15,0x16][j%2]}else{[0x05,0x06][j%2]}
    })
}

// Mansion block11 has a compact dense green crown above a gold pot, not
// the tall pale imported foliage. Only the complete native four-cell drawing
// owns this adaptation; edited/custom/other-family plants keep their paths.
fn native_mansion_plant(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    matches!(map,"CeladonMansion1F"|"CeladonMansion2F")
        &&p.kind==ModelKind::PottedPlant&&p.width==2&&p.height==2&&p.base_pixels==0.
        &&p.column+2<=g.width&&p.row+2<=g.height&&cells.len()==g.width*g.height
        &&p.indices(g.width).enumerate().all(|(j,i)| {
            let s=&cells[i].source;
            s.tileset_id.as_ref()=="mansion"&&s.metatile_id==0x11
                &&s.subtile_column==2+(j%2)as u8&&s.subtile_row==2+(j/2)as u8
                &&s.tile_index==[0x2e,0x2f,0x5e,0x5f][j]
        })
}
const MANSION_PLANT_PIXELS:[[usize;2];6]=[[1,3],[3,3],[2,0],[4,9],[5,9],[2,8]];

fn native_facility_plant(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if !matches!(map,"TrainerHouseB1F"|"SilphCo1F")||p.kind!=ModelKind::PottedPlant||p.width!=2||p.height!=4 {return false;}
    let first=&cells[p.row*g.width+p.column].source;
    if !matches!((first.metatile_id,first.subtile_column,first.subtile_row),(13,0,0)|(14,2,0)) {return false;}
    p.indices(g.width).enumerate().all(|(j,i)| {
        let s=&cells[i].source;
        s.tileset_id.as_ref()=="facility"&&s.metatile_id==first.metatile_id
            &&s.subtile_column==first.subtile_column+(j%2)as u8&&s.subtile_row==(j/2)as u8
            &&s.tile_index==[44,45,60,61,46,47,62,63][j]
    })
}

fn native_source_swatches<const N:usize>(p:&Placement,g:&GridGeometry,pixels:[[usize;2];N])->[[f32;2];N] {
    pixels.map(|[x,y]| {let uv=g.uv(p.column+x/8,p.row+y/8);
        [uv.0+(uv.1-uv.0)*((x%8)as f32+0.5)/8.,uv.2+(uv.3-uv.2)*((y%8)as f32+0.5)/8.]})
}
fn native_store_seat_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if !store_floor_map(map)||p.kind!=ModelKind::Cushion||p.base_pixels!=0.
        ||p.width!=2||!matches!(p.height,2|4)||cells.len()!=g.width*g.height
        ||p.column+2>g.width||p.row+p.height>g.height {return false;}
    let first=&cells[p.row*g.width+p.column].source;
    if !matches!(first.metatile_id,7|0x20)||!matches!(first.subtile_column,0|2)
        ||!matches!(first.subtile_row,0|2)||usize::from(first.subtile_row)+p.height>4 {return false;}
    let drawing=[[0x22,0x23],[0x32,0x33],[0x24,0x25],[0x34,0x35]];
    p.indices(g.width).enumerate().all(|(j,i)| {
        let s=&cells[i].source;let row=usize::from(first.subtile_row)+j/2;
        s.tileset_id.as_ref()=="mart"&&s.metatile_id==first.metatile_id
            &&s.subtile_column==first.subtile_column+(j%2)as u8&&usize::from(s.subtile_row)==row
            &&s.tile_index==drawing[row][j%2]
    })
}
fn native_store_aisle_shelf_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if !matches!(map,"GoldenrodDeptStore4F"|"CeladonDeptStore4F")
        ||p.kind!=ModelKind::RetailShelf||p.width!=2||p.height!=4||p.base_pixels!=0.
        ||cells.len()!=g.width*g.height||p.column+2>g.width||p.row+4>g.height {return false;}
    let first=&cells[p.row*g.width+p.column].source;
    matches!(first.subtile_column,0|2)&&first.subtile_row==0
        &&p.indices(g.width).enumerate().all(|(j,i)| {
            let s=&cells[i].source;
            s.tileset_id.as_ref()=="mart"&&s.metatile_id==first.metatile_id
                &&s.subtile_column==first.subtile_column+(j%2)as u8
                &&s.subtile_row==(j/2)as u8
                &&match first.metatile_id {
                    0x13=>crate::mart::display_rack_local(s)==Some(((j%2)as u8,(j/2)as u8)),
                    0x15=>s.tile_index==[[12,13],[80,81],[80,81],[94,95]][j/2][j%2],
                    _=>false,
                }
        })
}
fn native_store_plant_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    store_floor_map(map)&&p.kind==ModelKind::PottedPlant&&p.base_pixels==0.
        &&p.width==2&&p.height==4&&cells.len()==g.width*g.height
        &&p.column+2<=g.width&&p.row+4<=g.height
        &&p.indices(g.width).enumerate().all(|(j,i)| {
            let s=&cells[i].source;
            s.metatile_id==0x0b&&s.subtile_column==(j%2)as u8&&s.subtile_row==(j/2)as u8
                &&crate::mart::display_rack_local(s)==Some(((j%2)as u8,(j/2)as u8))
        })
}
fn native_store_shelf_neighbors(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->[bool;2] {
    if !native_store_aisle_shelf_source(map,p,cells,g) {return [false;2];}
    [p.column.checked_sub(p.width),p.column.checked_add(p.width)].map(|column| {
        column.is_some_and(|column| {
            let mut neighbor=p.clone();neighbor.column=column;
            native_store_aisle_shelf_source(map,&neighbor,cells,g)
        })
    })
}
fn native_lab_workstation_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<([[f32;2];8],[[f32;2];2])> {
    if !matches!(map,"OaksLab"|"ElmsLab")||p.kind!=ModelKind::LabWorkstation||p.width!=4||p.height!=3 {return None;}
    if !p.indices(g.width).enumerate().all(|(j,i)| {
        let s=&cells[i].source;let row=j/4;
        s.tileset_id.as_ref()=="lab"&&s.metatile_id==if row<2 {8}else{12}
            &&s.subtile_column==j as u8%4&&s.subtile_row==(2+row)as u8%4
            &&s.tile_index==[[10,11,12,13],[26,27,28,29],[37,38,38,39]][row][j%4]
    }) {return None;}
    Some((native_source_swatches(p,g,[[24,5],[9,9],[9,9],[0,1],[24,5],[4,4],[4,4],[0,1]]),
        native_source_swatches(p,g,[[2,17],[0,19]])))
}
fn native_elm_fixture_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<[[f32;2];4]> {
    if map!="ElmsLab"||p.width!=4||p.height!=4||p.kind!=ModelKind::LabRestorationMachine {return None;}
    let name="Lab restoration machine";
    let object=built_in_profiles().objects.iter().find(|o|o.name==name)?;
    if !p.indices(g.width).enumerate().all(|(j,i)| {
        let s=&cells[i].source;let y=j/4;let x=j%4;
        s.tileset_id.as_ref()=="lab"&&s.metatile_id==if p.kind==ModelKind::LabRestorationMachine&&y>=2{0x18}else{object.metatile}
            &&s.subtile_column==x as u8&&s.subtile_row==((object.origin[1]as usize+y)%4)as u8
            &&s.tile_index==object.tiles[y][x]
    }) {return None;}
    let [dark,shade,gray,white]=[[1,8],[0,8],[14,8],[14,9]];
    Some(native_source_swatches(p,g,[dark,shade,gray,white]))
}
fn append_native_elm_healing_machine(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,
    b:[f32;4],swatches:[[f32;2];4]) {
    let [w,e,_,s]=b;let scale=g.tile_height/8.;let n=s-24.*scale;
    let ring=|inset:f32| {
        let [w,e,n,s]=[w+inset*scale,e-inset*scale,n+inset*scale,s-inset*scale];
        let c=0.8*scale;
        [[w+c,n],[e-c,n],[e,n+c],[e,s-c],[e-c,s],[w+c,s],[w,s-c],[w,n+c]]
    };
    let mut triangle=|points:[[f32;3];3],uv:[f32;2]| {
        let [a,b,c]=points.map(Vec3::from_array);let cross=(b-a).cross(c-a);
        if cross.length_squared()<0.000001 {return;}
        let normal=cross.normalize();let first=out.positions.len()as u32;
        out.positions.extend(points);out.normals.extend([normal.to_array();3]);out.uvs.extend([uv;3]);
        out.colors.extend([crate::interior_models::authored_face_color([1.;4],normal);3]);
        out.indices.extend([first,first+1,first+2]);
    };
    // Low rounded chassis: the source's last eight rows are its front, not a
    // tall pair of cylinders. Beveled rings avoid hard block-shaped corners.
    for (a,ya,b,yb,uv) in [(ring(0.4),0.,ring(0.),0.4,swatches[1]),
        (ring(0.),0.4,ring(0.),7.4,swatches[2]),(ring(0.),7.4,ring(0.6),8.,swatches[3])] {
        for i in 0..8 {
            if i==4&&ya==0.4 {continue;}
            let j=(i+1)%8;
            let points=[[b[j][0],yb*scale,b[j][1]],[b[i][0],yb*scale,b[i][1]],
                [a[i][0],ya*scale,a[i][1]],[a[j][0],ya*scale,a[j][1]]];
            // Reverse winding so all exterior normals point out/up.
            triangle([points[0],points[2],points[1]],uv);triangle([points[0],points[3],points[2]],uv);
        }
    }
    let outer=ring(0.6);
    let inner=[[w+scale,n+scale],[e-scale,n+scale],[e-scale,n+scale],[e-scale,s-scale],
        [e-scale,s-scale],[w+scale,s-scale],[w+scale,s-scale],[w+scale,n+scale]];
    for i in 0..8 {let j=(i+1)%8;
        let p=|v:[f32;2]|[v[0],8.*scale,v[1]];
        triangle([p(outer[j]),p(outer[i]),p(inner[i])],swatches[3]);
        triangle([p(outer[j]),p(inner[i]),p(inner[j])],swatches[3]);
    }
    drop(triangle);
    let (left,right,north,south)=(w+scale,e-scale,n+scale,s-scale);
    let x_at=|x:f32|left+(right-left)*x/32.;let z_at=|y:f32|north+(south-north)*y/24.;
    for y in 0..3 {for x in 0..4 {
        let low=if matches!(x,1|2)&&y>=1 {if y==1 {continue;}23.}else{y as f32*8.};
        let high=(y+1)as f32*8.;let uv=g.uv(p.column+x,p.row+y);
        let v0=uv.2+(uv.3-uv.2)*(low-y as f32*8.)/8.;
        append_top(out,[x_at(x as f32*8.),x_at((x+1)as f32*8.),z_at(low),z_at(high)],8.*scale,
            (uv.0,uv.1,v0,uv.3));
    }}
    let bay=[x_at(8.),x_at(24.),z_at(8.),z_at(23.)];
    let first=out.positions.len();append_top(out,bay,5.5*scale,(0.,0.,0.,0.));out.uvs[first..].fill(swatches[0]);
    let [a,b,c,d]=bay;
    for (points,normal) in [
        ([[a,5.5*scale,c],[b,5.5*scale,c],[b,8.*scale,c],[a,8.*scale,c]],Vec3::Z),
        ([[b,5.5*scale,d],[a,5.5*scale,d],[a,8.*scale,d],[b,8.*scale,d]],Vec3::NEG_Z),
        ([[a,5.5*scale,d],[a,5.5*scale,c],[a,8.*scale,c],[a,8.*scale,d]],Vec3::X),
        ([[b,5.5*scale,c],[b,5.5*scale,d],[b,8.*scale,d],[b,8.*scale,c]],Vec3::NEG_X)] {
        append_quad(out,points,normal.to_array(),[swatches[1];4],
            crate::interior_models::authored_face_color([1.;4],normal));
    }
    for x in 0..4 {
        let uv=g.uv(p.column+x,p.row+3);let a=w+0.8*scale+(e-w-1.6*scale)*x as f32/4.;
        let b=w+0.8*scale+(e-w-1.6*scale)*(x+1)as f32/4.;
        append_quad(out,[[b,0.4*scale,s],[b,7.4*scale,s],[a,7.4*scale,s],[a,0.4*scale,s]],
            Vec3::Z.to_array(),[[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2],[uv.0,uv.3]],
            crate::interior_models::authored_face_color([1.;4],Vec3::Z));
    }
}
fn native_elm_central_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<[[f32;2];8]> {
    if map!="ElmsLab"||p.kind!=ModelKind::LabCounter||p.width!=4||p.height!=4 {return None;}
    let object=built_in_profiles().objects.iter().find(|o|o.name=="Lab central desk assembly")?;
    if !p.indices(g.width).enumerate().all(|(j,i)| {let s=&cells[i].source;
        s.tileset_id.as_ref()=="lab"&&s.metatile_id==0x21&&s.subtile_column==(j%4)as u8
            &&s.subtile_row==(j/4)as u8&&s.tile_index==object.tiles[j/4][j%4]
    }) {return None;}
    // These source tiles select VRAM bank one: raw81 is PNG61, not PNG81.
    let [dark,shade,gray,white]=[[18,8],[28,8],[0,8],[19,8]];
    Some(native_source_swatches(p,g,[shade,gray,gray,white,gray,dark,dark,white]))
}
fn append_native_elm_desk(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4],swatches:[[f32;2];8]) {
    let [w,e,_,s]=b;let scale=g.tile_height/8.;let n=s-16.*scale;let top=10.*scale;
    let first=out.positions.len();let indices=out.indices.len();
    architecture_box(out,[w,e,0.,top,n,s],[1.;3]);out.uvs[first..].fill(swatches[2]);
    out.indices.drain(indices..indices+12);
    // Source rows12..27 describe the phone/console worktop; the final four
    // pixels describe its dark front apron. Floor and the figure above it
    // are not stretched onto the physical desk or turned into a chair.
    for (a,b) in [(12,16),(16,24),(24,28)] {for x in 0..4 {
        let uv=g.uv(p.column+x,p.row+a/8);
        let v0=uv.2+(uv.3-uv.2)*(a%8)as f32/8.;
        let v1=v0+(uv.3-uv.2)*(b-a)as f32/8.;
        append_top(out,[w+(e-w)*x as f32/4.,w+(e-w)*(x+1)as f32/4.,
            n+(a-12)as f32*scale,n+(b-12)as f32*scale],top,(uv.0,uv.1,v0,v1));
    }}
    for x in 0..4 {
        let uv=g.uv(p.column+x,p.row+3);let a=w+(e-w)*x as f32/4.;let b=w+(e-w)*(x+1)as f32/4.;
        append_quad(out,[[b,0.,s],[b,top,s],[a,top,s],[a,0.,s]],[0.,0.,1.],
            [[uv.1,uv.3],[uv.1,(uv.2+uv.3)*0.5],[uv.0,(uv.2+uv.3)*0.5],[uv.0,uv.3]],
            crate::interior_models::authored_face_color([1.;4],Vec3::Z));
    }
    let mut piece=|rect:[f32;6],uv:[f32;2]| {
        let first=out.positions.len();
        architecture_box(out,[w+rect[0]*scale,w+rect[1]*scale,top+rect[2]*scale,top+rect[3]*scale,
            n+rect[4]*scale,n+rect[5]*scale],[1.;3]);out.uvs[first..].fill(uv);
    };
    piece([3.,12.,0.,0.8,3.,10.],swatches[2]);
    for r in [[4.,11.,0.8,1.5,3.,5.],[3.,5.,0.8,1.5,4.,8.],[10.,12.,0.8,1.5,4.,8.]] {piece(r,swatches[0]);}
    piece([18.,30.,0.,1.3,2.,12.],swatches[2]);
    piece([20.,28.,1.3,1.5,3.,6.],swatches[5]);
    for y in 0..2 {for x in 0..5 {
        piece([19.+x as f32*1.8,20.+x as f32*1.8,1.3,1.5,8.+y as f32*1.5,9.+y as f32*1.5],swatches[3]);
    }}
}
fn native_mansion_workstation_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<[[f32;2];8]> {
    let block=cells[p.row*g.width+p.column].source.metatile_id;
    if p.kind!=ModelKind::LabWorkstation || p.width!=4 || p.height!=4
        || !p.indices(g.width).enumerate().all(|(j,i)| {
            let s=&cells[i].source;
            let expected=match (block,j/4>=2,j%4>=2) {
                (0x12,false,_)=>0x12,(0x12,true,_)=>0x13,
                (0x18,false,false)=>0x18,(0x18,false,true)=>0x19,
                (0x18,true,false)=>0x1b,(0x18,true,true)=>0x1a,
                (0x1c,_,_)=>0x1c,_=>return false,
            };
            s.metatile_id==expected && crate::house::mansion_workstation_local(map,s)
                ==Some(((j%4)as u8,(j/4)as u8,4,4))
        }) {return None;}
    Some(native_source_swatches(p,g,[[20,5],[1,2],[3,10],[1,1],[9,10],[0,1],[21,6],[24,5]]))
}
const MANSION_CLOCK_PIXELS:[[usize;2];4]=[[1,8],[3,5],[7,12],[3,10]];
fn native_mansion_clock_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<[[f32;2];4]> {
    if !matches!(map,"CeladonMansion1F"|"CeladonMansion2F"|"CeladonMansion3F")
        ||p.kind!=ModelKind::PendulumClock||p.width!=2||p.height!=4||p.base_pixels!=0.
        ||p.column+2>g.width||p.row+4>g.height||cells.len()!=g.width*g.height{return None;}
    p.indices(g.width).enumerate().all(|(j,i)| {
        let s=&cells[i].source;
        s.tileset_id.as_ref()=="mansion"&&s.metatile_id==0xf
            &&s.subtile_column==2+(j%2)as u8&&s.subtile_row==(j/2)as u8
            &&s.tile_index==[0x36,0x38,0x24,0x25,0x34,0x35,0x32,0x33][j]
    }).then(||native_source_swatches(p,g,MANSION_CLOCK_PIXELS))
}
fn native_mansion_corridor_furniture(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    matches!(map,"CeladonMansion1F"|"CeladonMansion2F"|"CeladonMansion3F")
        &&cells[p.row*g.width+p.column].source.tileset_id.as_ref()=="mansion"
        &&(native_bookcase_source(map,p,cells,g).is_some()
            ||native_mansion_workstation_source(map,p,cells,g).is_some()
            ||native_mansion_clock_source(map,p,cells,g).is_some())
}

fn native_link_seat_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<[[f32;2];4]> {
    if p.kind!=ModelKind::LinkSeat || p.width!=2 || p.height!=2 {return None;}
    let block=cells[p.row*g.width+p.column].source.metatile_id;
    if p.indices(g.width).enumerate().all(|(j,i)|cells[i].source.metatile_id==block
        && crate::house::mansion_office_seat_local(map,&cells[i].source)
            ==Some(((j%2)as u8,(j/2)as u8,2,2))) {
        return Some(native_source_swatches(p,g,[[4,4],[6,5],[5,11],[5,4]]));
    }
    if !kanto_native_floor_map(map) || !map.ends_with("Pokecenter2FBeta") {return None;}
    let source=&cells[p.row*g.width+p.column].source;
    if source.subtile_row!=0 || !matches!(source.subtile_column,0|2) || !matches!(source.metatile_id,0x29|0x2d) {return None;}
    if !p.indices(g.width).enumerate().all(|(j,i)| {let s=&cells[i].source;
        s.tileset_id.as_ref()=="pokecenter" && s.metatile_id==source.metatile_id
            && s.subtile_column==source.subtile_column+(j%2)as u8 && s.subtile_row==(j/2)as u8
            && s.tile_index==[0x48,0x49,0x58,0x59][j]}) {return None;}
    Some(native_source_swatches(p,g,[[4,3],[3,3],[2,11],[0,2]]))
}
fn reviewed_arcade_room(map:&str)->bool {matches!(map,"CeladonGameCorner"|"GoldenrodGameCorner")}

fn native_arcade_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<[[f32;2];9]> {
    if !reviewed_arcade_room(map) {return None;}
    let s=&cells[p.row*g.width+p.column].source;
    if s.tileset_id.as_ref()!="game_corner" {return None;}
    let (ox,oy,drawing,pixels): (u8,u8,Vec<u16>,[[usize;2];9])=match (p.kind,s.metatile_id,p.width,p.height) {
        (ModelKind::ArcadePair,7|11,4,2) if s.subtile_column==0
            && (s.subtile_row==0 || s.metatile_id==7 && s.subtile_row==2)=>
            (0,s.subtile_row,vec![0xa0,0xa1,0xa2,0xa3,0x90,0x91,0x92,0x93],
                [[6,1],[1,0],[0,0],[0,0],[3,1],[3,1],[14,0],[0,0],[0,0]]),
        (ModelKind::ArcadeMachine,0x0e,2,4) if matches!(s.subtile_column,0|2) && s.subtile_row==0=>
            (s.subtile_column,0,vec![0xa6,0xa7,0x22,0x23,0x2a,0x2b,0x3a,0x3b],
                [[14,2],[0,0],[1,0],[1,0],[0,0],[0,0],[14,2],[1,0],[1,0]]),
        (ModelKind::ArcadeMachine,8,2,2) if matches!(s.subtile_column,0|2) && s.subtile_row==2=> {
            let tiles=if s.subtile_column==0 {vec![0x80,0x81,0x90,0x91]}else{vec![0x82,0x83,0x92,0x93]};
            let orange=if s.subtile_column==0 {[2,1]}else{[1,2]};
            let red=if s.subtile_column==0 {[14,2]}else{[13,1]};
            (s.subtile_column,2,tiles,[[0,0],orange,[2,0],[2,0],[0,0],[0,0],red,[2,0],[2,0]])
        },_=>return None,
    };
    if !p.indices(g.width).enumerate().all(|(j,i)| {let c=&cells[i].source;
        c.tileset_id==s.tileset_id && c.metatile_id==s.metatile_id
            && c.subtile_column==ox+(j%p.width)as u8 && c.subtile_row==oy+(j/p.width)as u8
            && c.tile_index==drawing[j]}) {return None;}
    Some(pixels.map(|[x,y]| {let uv=g.uv(p.column+x/8,p.row+y/8);
        [uv.0+(uv.1-uv.0)*((x%8)as f32+0.5)/8.,uv.2+(uv.3-uv.2)*((y%8)as f32+0.5)/8.]}))
}
pub(super) fn trace_arcade_palette(map:&str,cells:&[&VisualTile],g:&GridGeometry,
    placements:&[Placement],images:&TerrainImageSamples) {
    if std::env::var_os("CRYSTAL_ARCADE_PALETTE_TRACE").is_none() || !reviewed_arcade_room(map) {return;}
    let mut seen=std::collections::BTreeSet::new();
    for p in placements.iter().filter(|p|native_arcade_source(map,p,cells,g).is_some()) {
        let source=&cells[p.row*g.width+p.column].source;
        let key=format!("{:?}:{}:{}:{}",p.kind,source.metatile_id,source.subtile_column,source.subtile_row);
        if !seen.insert(key.clone()) {continue;}
        let sample=|x:usize,y:usize| {
            let tile=cells[(p.row+y/8)*g.width+p.column+x/8];
            let rgba=tile_rgba(images,tile).ok()?;let at=((y%8)*8+x%8)*4;
            rgba.get(at..at+4).map(|v|[v[0],v[1],v[2],v[3]])
        };
        let roles=native_arcade_source(map,p,cells,g).unwrap().map(|uv|[
            ((uv[0]*g.width as f32-p.column as f32)*8.-0.5).round().max(0.)as usize,
            ((uv[1]*g.height as f32-p.row as f32)*8.-0.5).round().max(0.)as usize]);
        eprintln!("arcade-palette map={map} kind={key} roles={:?}",roles.map(|[x,y]|sample(x,y)));
        let mut colors=std::collections::BTreeMap::new();
        for y in 0..p.height*8 {for x in 0..p.width*8 {if let Some(color)=sample(x,y) {
            colors.entry(color).or_insert((x,y));
        }}}
        eprintln!("arcade-palette map={map} kind={key} source-colors={colors:?}");
    }
}

fn native_center_counter_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<[[f32;2];5]> {
    if !kanto_native_floor_map(map) || !map.contains("Pokecenter")
        || p.kind!=ModelKind::ReceptionCounter || p.height!=2 {return None;}
    let s=&cells[p.row*g.width+p.column].source;
    if s.tileset_id.as_ref()!="pokecenter" || s.subtile_column!=0 || s.subtile_row!=0 {return None;}
    let top:&[u16]=match s.metatile_id {5=>&[0x34;4],6=>&[0x0c,0x0c,0x34,0x34],7=>&[0x34,0x0c],0x38=>&[0x34;2],_=>return None};
    if p.width!=top.len() || !p.indices(g.width).enumerate().all(|(j,i)| {let c=&cells[i].source;
        c.tileset_id==s.tileset_id && c.metatile_id==s.metatile_id
            && c.subtile_column==(j%p.width)as u8 && c.subtile_row==(j/p.width)as u8
            && c.tile_index==if j<p.width {top[j]}else{0x24}}) {return None;}
    let cap=top.iter().position(|&t|t==0x34).unwrap();
    Some(native_source_swatches(p,g,[[1,12],[0,10],[cap*8+2,4],[cap*8+2,4],[1,9]]))
}
pub(super) fn mart_counter_swatches(g:&GridGeometry,index:usize,tile:u16)->Option<[[f32;2];5]> {
    let (cream,gold,ink)=match tile {
        0x1e=>([3,2],[3,6],[3,7]),
        0x2e|0x3e=>([3,2],[1,2],[0,2]),
        0x2f|0x3f=>([3,2],[6,2],[7,2]),
        0x20=>([2,0],[1,0],[0,0]),0x21=>([0,0],[6,0],[7,0]),_=>return None,
    };
    let uv=g.uv(index%g.width,index/g.width);
    let sample=|[x,y]:[usize;2]|[uv.0+(uv.1-uv.0)*(x as f32+0.5)/8.,uv.2+(uv.3-uv.2)*(y as f32+0.5)/8.];
    Some([cream,ink,gold,cream,gold].map(sample))
}
fn native_mart_counter_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<[[f32;2];5]> {
    if !(kanto_native_floor_map(map)||store_floor_map(map)&&!map.ends_with("Elevator"))
        || p.kind!=ModelKind::ReceptionCounter || p.height!=2 {return None;}
    let s=&cells[p.row*g.width+p.column].source;
    if s.tileset_id.as_ref()!="mart" {return None;}
    let (origin,top,front): (u8,&[u16],&[u16])=match s.metatile_id {
        0x0c=>(2,&[0x2e,0x1e],&[0x18,0x1a]),
        0x0d=>(0,&[0x1e;4],&[0x1a;4]),
        0x0e=>(0,&[0x1e,0x2f],&[0x1a,0x19]),
        0x22=>(0,&[0x20,0x21],&[0x30,0x31]),_=>return None,
    };
    if p.width!=top.len() || !p.indices(g.width).enumerate().all(|(j,i)| {
        let c=&cells[i].source;let y=j/p.width;let x=j%p.width;
        c.tileset_id==s.tileset_id && c.metatile_id==s.metatile_id
            && c.subtile_column==origin+x as u8 && c.subtile_row==y as u8
            && c.tile_index==if y==0 {top[x]}else{front[x]}
    }) {return None;}
    mart_counter_swatches(g,p.row*g.width+p.column,s.tile_index)
}
fn native_station_ticket_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    p.kind==ModelKind::VendingMachine && p.width==2 && p.height==8
        && p.indices(g.width).enumerate().all(|(j,i)|crate::train_station::ticket_box_local(map,&cells[i].source)
            ==Some(((j%2)as u8,(j/2)as u8,2,8)))
}
fn native_store_vending_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    if g.width.checked_mul(g.height)!=Some(cells.len())||p.column+2>g.width||p.row+4>g.height {return false;}
    let origin=cells[p.row*g.width+p.column].source.subtile_column;
    matches!(map,"GoldenrodDeptStore6F"|"CeladonDeptStore6F")
        &&p.kind==ModelKind::VendingMachine&&p.width==2&&p.height==4&&p.base_pixels==0.
        &&matches!(origin,0|2)
        &&p.indices(g.width).enumerate().all(|(j,i)| {
            let s=&cells[i].source;
            s.metatile_id==0x21&&s.subtile_column==origin+(j%2)as u8&&s.subtile_row==(j/2)as u8
                &&crate::mart::display_rack_local(s)==Some(((j%2)as u8,(j/2)as u8))
        })
}
fn native_station_wall_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    let block=cells[p.row*g.width+p.column].source.metatile_id;
    p.kind==ModelKind::WallClinical && p.width==4 && p.height==8
        && p.indices(g.width).enumerate().all(|(j,i)|cells[i].source.metatile_id==if j<16 {block}else{0x0b}
            && crate::train_station::wall_local(map,&cells[i].source)==Some(((j%4)as u8,(j/4)as u8,4,8)))
}

fn native_station_chair_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    p.kind==ModelKind::Chair && p.width==2 && p.height==2 && p.indices(g.width).enumerate().all(|(j,i)|
        crate::train_station::seat_local(map,&cells[i].source)==Some(((j%2)as u8,(j/2)as u8)))
}

/// Append only after successful resolution. The original immutable ground
/// atlas supplies every vacated pixel, with no fabricated floor or collision.
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    p: &Placement,
    claimed: &mut [bool],
    world_origin: [i32; 2],
) -> bool {
    append_with_tables(mesh,map,cells,geometry,p,claimed,world_origin,&[])
}

/// Production supplies the complete resolved furnishings, so a seat can face
/// its actual adjacent table without guessing from an individual tile's paint.
pub(super) fn append_with_tables(
    mesh:&mut TerrainMeshData,map:&str,cells:&[&VisualTile],geometry:&GridGeometry,
    p:&Placement,claimed:&mut[bool],world_origin:[i32;2],tables:&[Placement],
) -> bool {
    if p.indices(geometry.width).any(|i| claimed[i]) {
        return false;
    }
    let Some(selected) = crate::interior_models::selected_model(p.kind) else {
        append_missing_source(mesh, geometry, p, claimed);
        return true;
    };
    let piece_source=&cells[p.row*geometry.width+p.column].source;
    if (native_clinic_fixtures::station(map,p,cells,geometry) && !selected.has_native_television_materials())
        || (native_clinic_fixtures::stair(map,p,cells,geometry).is_some() && !selected.has_native_stair_materials()) {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if native_classroom::matches(map,p,cells,geometry)&&!native_classroom::materials(p.kind,selected) {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if native_radio_room::matches(map,p,cells,geometry)&&p.kind==ModelKind::PottedPlant
        &&!selected.has_native_safari_plant_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if native_gift_shop::traditional_table(map,p,cells,geometry)
        && !selected.has_native_low_table_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if native_small_display_source(map,p,cells,geometry)
        && (!selected.has_native_low_table_materials()
            || crate::interior_models::selected_model(ModelKind::FlowerStand).is_none_or(|flower|!flower.has_native_display_flower_materials())) {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if native_lab_workstation_source(map,p,cells,geometry).is_some() && !selected.has_native_workstation_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if native_mansion_workstation_source(map,p,cells,geometry).is_some()
        && (!selected.has_native_workstation_materials()
            || crate::interior_models::selected_model(ModelKind::LinkSeat).is_none_or(|seat|!seat.has_native_link_seat_materials()))
        || native_link_seat_source(map,p,cells,geometry).is_some() && !selected.has_native_link_seat_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if native_arcade_source(map,p,cells,geometry).is_some() && !selected.has_native_arcade_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if (native_mart_counter_source(map,p,cells,geometry).is_some()
        || native_center_counter_source(map,p,cells,geometry).is_some()) && !selected.has_native_store_counter_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if (native_station_ticket_source(map,p,cells,geometry)||native_store_vending_source(map,p,cells,geometry))
        && !selected.has_native_ticket_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if (native_station_chair_source(map,p,cells,geometry)||native_radio_room::chair_source(map,p,cells,geometry))
        && !selected.has_native_station_chair_material() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if native_station_plant_source(map,p,cells,geometry) && (!selected.has_native_station_plant_materials()
        || !model(ModelKind::FlowerStand).has_native_station_bloom_materials()) {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if reviewed_native_room(map) && native_red_bed_source(p,cells,geometry)
        && !selected.has_native_bed_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if reviewed_native_room(map) && native_house_radio_source(p,cells,geometry)
        && (!selected.has_native_house_radio_materials()
            || crate::interior_models::selected_model(ModelKind::DrawerCabinet)
                .is_none_or(|m|!m.has_native_home_fixture_materials())) {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if reviewed_native_room(map) && native_home_fixture_source(p, cells, geometry).is_some()
        && !selected.has_native_home_fixture_materials() {
        append_missing_source(mesh, geometry, p, claimed);
        return true;
    }
    if reviewed_native_room(map) && native_computer_source(p,cells,geometry).is_some()
        && (!selected.has_native_computer_materials()
            || piece_source.tileset_id.as_ref()=="house" && crate::interior_models::selected_model(ModelKind::DrawerCabinet)
                .is_none_or(|m|!m.has_native_home_fixture_materials())) {
        append_missing_source(mesh,geometry,p,claimed);
        return true;
    }
    if native_pokecom_equipment::computer(map,p,cells,geometry) && !selected.has_native_computer_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if (native_home_stair::source(map,p,cells,geometry).is_some()
        ||native_mansion_architecture::stair(map,p,cells,geometry).is_some()
        ||native_radio_stair::stair(map,p,cells,geometry).is_some())&&!selected.has_native_stair_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if native_mansion_clock_source(map,p,cells,geometry).is_some()&&!selected.has_native_gift_fixture_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if native_bookcase_source(map,p,cells,geometry).is_some()
        && selected.native_bookcase_structural_mask().is_none() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if reviewed_native_room(map) && native_house_television_source(p,cells,geometry).is_some()
        && !selected.has_native_television_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if (native_house_dining_table(map,cells,geometry,p)||native_radio_studio::table(map,p,cells,geometry)) && !selected.has_native_dining_table_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if p.kind==ModelKind::TrashCan && !selected.has_native_trashcan_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if native_gift_shop::traditional_cushion(map,p,cells,geometry)
        && !selected.has_native_gift_fixture_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if native_store_seat_source(map,p,cells,geometry)&&!selected.has_native_gift_fixture_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if let Some(kind)=native_gift_shop::source(map,p,cells,geometry) {
        if traditional_wood_wall::source(map,p,cells,geometry)&&kind==ModelKind::RadioWall
            &&crate::interior_models::selected_model(ModelKind::Radio)
                .is_none_or(|radio|!radio.has_native_home_fixture_materials()) {
            append_missing_source(mesh,geometry,p,claimed);return true;
        }
        if (matches!(kind,ModelKind::PendulumClock|ModelKind::Cushion)&&!selected.has_native_gift_fixture_materials())
            ||(matches!(kind,ModelKind::Cabinet|ModelKind::DrawerCabinet)&&!selected.has_native_home_fixture_materials()) {
            append_missing_source(mesh,geometry,p,claimed);return true;
        }
    }
    if p.kind==ModelKind::PottedPlant&&(native_house_plant::source(map,p,cells,geometry)||native_mansion_plant(map,p,cells,geometry)||native_safari_office::matches(map,p,cells,geometry)||native_facility_plant(map,p,cells,geometry)||native_port_plant(map,p,cells,geometry)||native_store_plant_source(map,p,cells,geometry)||native_gate_plant_source(map,p,cells,geometry))
        &&!selected.has_native_safari_plant_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if p.kind==ModelKind::LongPlanter&&native_gate_plant_source(map,p,cells,geometry)
        &&!selected.has_native_station_plant_materials() {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if (native_radio_studio::computer(map,p,cells,geometry)&&!selected.has_native_radio_terminal_materials())
        || (native_radio_studio::equipment_size(map,p,cells,geometry).is_some()
            &&!native_radio_studio::computer(map,p,cells,geometry)&&!selected.has_native_radio_equipment_materials()) {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if native_radio_studio::cap(map,p,cells,geometry)&&!native_radio_studio::cap_ready(map,p,cells,geometry,tables) {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if native_radio_studio::grille_foot(map,p,cells,geometry)&&!native_radio_studio::grille_foot_ready(map,p,cells,geometry,tables) {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    if native_radio_studio::side_missing_context(map,p,cells,geometry,tables) {
        append_missing_source(mesh,geometry,p,claimed);return true;
    }
    let source = &cells[p.ground];
    let ground_uv = geometry.uv(source.column as usize, source.row as usize);
    for i in p.indices(geometry.width) {
        if traditional_wood_wall::source(map,p,cells,geometry)&&i/geometry.width<p.row+2 {continue;}
        if traditional_wood_wall::source(map,p,cells,geometry) {
            let floor=geometry.uv(p.column+2,p.row+2);
            let field=[floor.0+(floor.1-floor.0)*0.5/8.,floor.2+(floor.3-floor.2)*2.5/8.];
            native_house_wood_floor_field(&mut mesh.textured,
                geometry.bounds(i%geometry.width,i/geometry.width).into(),0.,
                [i as i32%geometry.width as i32,i as i32/geometry.width as i32],field);
            continue;
        }
        // The tall source wall is a perspective drawing, not eight rows of
        // walkable floor behind the physical rear wall. Retain its foot course.
        if native_station_wall_source(map,p,cells,geometry)
            && i/geometry.width<p.row+p.height-1 {continue;}
        let (x0, x1, z0, z1) = geometry.bounds(i % geometry.width, i / geometry.width);
        append_top(
            &mut mesh.textured,
            [x0, x1, z0, z1],
            if p.kind == ModelKind::StairFlight {
                p.base_pixels * geometry.tile_height / 8.0
            } else {
                0.0
            },
            if p.kind==ModelKind::StairFlight&&cells[i].source.tileset_id.as_ref()=="facility"
                &&native_home_stair::source(map,p,cells,geometry).is_some() {
                // Continue the authored two-color checker under raised flights.
                let s=&cells[i].source;
                let tile=if (s.subtile_column+s.subtile_row)%2==0 {1}else{0x26};
                ground_sample(cells,map,"facility",tile).map(|i|
                    geometry.uv(i%geometry.width,i/geometry.width)).unwrap_or(ground_uv)
            }else if let Some(uv)=native_radio_stair::well_uv(map,p,cells,geometry) {uv}
            else{native_radio_studio::floor_sample(map,p,cells,geometry,i).map(|j|geometry.uv(j%geometry.width,j/geometry.width)).unwrap_or(ground_uv)},
        );
    }
    let west = geometry.origin_x + p.column as f32 * geometry.tile_width;
    let east = west + p.width as f32 * geometry.tile_width;
    let piece_source = &cells[p.row * geometry.width + p.column].source;
    // The arcade's one- and two-row drawings are one north wall course.
    // Drawing height must not turn its panels into staggered freestanding posts.
    let arcade_poster = p.kind == ModelKind::PictureFrame
            && piece_source.tileset_id.as_ref() == "game_corner"
            && piece_source.metatile_id == 0x28;
    let arcade_wall_plane = p.kind == ModelKind::ArcadeWall || arcade_poster;
    let native_gate_wall=kanto_native_floor_map(map) && p.kind==ModelKind::WallClinical
        && piece_source.tileset_id.as_ref()=="gate" && (1..=3).contains(&p.height)
        && piece_source.subtile_row==0;
    let native_gate_doors=kanto_native_floor_map(map) && p.kind==ModelKind::GateDoorFrame
        && p.width==4 && p.height==3 && p.indices(geometry.width).enumerate().all(|(j,i)| {
            let s=&cells[i].source;
            s.tileset_id.as_ref()=="gate" && s.metatile_id==4
                && s.subtile_column==j as u8%4 && s.subtile_row==j as u8/4
                && s.tile_index==[2,3,2,3,0x12,0x13,0x12,0x13,0x11,0x11,0x11,0x11][j]
        });
    let native_cafe_wall=map=="CeladonCafe"&&p.kind==ModelKind::ArcadeWall
        && piece_source.tileset_id.as_ref()=="game_corner"&&p.height<=2
        && matches!(piece_source.metatile_id,2|0x13|0x28|0x29|0x2d);
    let native_player_wall=reviewed_native_room(map) && p.kind==ModelKind::WallDomestic
        && piece_source.tileset_id.as_ref()=="players_house"
        && p.indices(geometry.width).all(|i|cells[i].source.tile_index==0x11);
    let south_row = if native_radio_studio::panel(map,p,cells,geometry) {p.row+4}
    else if p.kind==ModelKind::WallAcoustic&&native_radio_room::matches(map,p,cells,geometry) {
        p.row+2
    } else if p.kind==ModelKind::ArcadeWall&&native_safari_office::matches(map,p,cells,geometry) {
        p.row+2-usize::from(piece_source.subtile_row)
    } else if native_gift_shop::source(map,p,cells,geometry)==Some(ModelKind::GiftShelf) {
        p.row+2
    } else if native_gift_shop::source(map,p,cells,geometry)==Some(ModelKind::LabCounter) {
        p.row+3
    } else if native_clinic::cabinet_source(map,p,cells,geometry) {
        p.row+4
    } else if native_clinic::wall_source(map,p,cells,geometry) && p.height==1 {
        p.row+2
    } else if native_mansion_workstation_source(map,p,cells,geometry).is_some() {
        p.row+2
    } else if native_cafe_wall {
        p.row-usize::from(piece_source.subtile_row)+2
    } else if native_house_dining_table(map,cells,geometry,p) {
        p.row+native_dining_table_rows(p)
    } else if native_player_wall {
        p.row-usize::from(piece_source.subtile_row)+2
    } else if native_gate_wall {
        // The authored upper fragments carry a 16px forward offset; two-row
        // planter fragments carry 8px. All join the same three-row course.
        p.row+3
    } else if arcade_wall_plane {
        p.row - usize::from(piece_source.subtile_row) + 1
    } else if p.kind==ModelKind::WallTraditional && piece_source.tileset_id.as_ref()=="tower"
        && piece_source.tile_index==0x41 && p.depth_pixels>p.height as f32*8. {
        p.row+(p.depth_pixels/8.) as usize
    } else {
        p.row + p.height
    };
    let poster_relief = if arcade_poster { 0.2 * geometry.tile_height / 8.0 } else { 0.0 };
    let mut south = geometry.origin_z + south_row as f32 * geometry.tile_height + poster_relief;
    let mut north = south - p.depth_pixels * geometry.tile_height / 8.0;
    if p.height==2 && matches!(p.kind,ModelKind::GameConsole|ModelKind::Stool|ModelKind::ArcadeStool
        |ModelKind::Chair|ModelKind::LinkSeat) {
        let center=geometry.origin_z+(p.row as f32+
            if native_radio_room::chair_source(map,p,cells,geometry) {1.5}else{1.})*geometry.tile_height;
        north=center-p.depth_pixels*geometry.tile_height/16.;
        south=center+p.depth_pixels*geometry.tile_height/16.;
    }
    let inset = if native_radio_stair::stair(map,p,cells,geometry).is_some() || native_radio_studio::computer(map,p,cells,geometry) || native_radio_studio::handled(map,p,cells,geometry)&&p.kind!=ModelKind::ArcadeStool || native_radio_room::matches(map,p,cells,geometry)&&p.kind!=ModelKind::PottedPlant || matches!(
        p.kind,
        ModelKind::CarpetCloth
            | ModelKind::PrizeBooth
            | ModelKind::WallPanel
            | ModelKind::WindowWall
            | ModelKind::GateDoorFrame
            | ModelKind::RadioWall
            | ModelKind::ClinicalPartition
            | ModelKind::TimberPartition
            | ModelKind::ReceptionCounter
            | ModelKind::LabCounter
            | ModelKind::WallDomestic
            | ModelKind::WallTraditional
            | ModelKind::WallClinical
            | ModelKind::WallAcoustic
            | ModelKind::ArcadeWall
    ) {
        0.0
    } else {
        geometry.tile_width * 0.045
    };
    // Adjacent native window/bookcase units retain their own proportions.
    // Whole-drawing claims do not license stretching one window across a wall.
    // The guarded 2f/30 lower band is already covered by its complete upper
    // course. Retain its source claim/underlay without a duplicate wall volume.
    let repeats = if native_cafe_wall || native_safari_office::matches(map,p,cells,geometry) {1} else if p.kind == ModelKind::ArcadeWall && piece_source.subtile_row == 1 {
        debug_assert!(matches!(piece_source.metatile_id, 0x2f | 0x30));
        0
    } else if (matches!(
        p.kind,
        ModelKind::WindowWall | ModelKind::Bookcase | ModelKind::LabBookcase | ModelKind::ArcadeWall
    ) || (p.kind == ModelKind::PictureFrame
        && cells[p.row * geometry.width + p.column]
            .source
            .tileset_id
            .as_ref()
            == "pokecenter"))
        && p.width % 2 == 0
    {
        p.width / 2
    } else {
        1
    };
    let corridor_furniture_first=mesh.textured.positions.len();
    let object_solid_first=mesh.solid.positions.len();
    native_radio_room::append_bookcase_backing(&mut mesh.textured,geometry,p,cells,world_origin,map);
    for unit in 0..repeats {
        let mut left = west + (east - west) * unit as f32 / repeats as f32;
        let mut right = west + (east - west) * (unit + 1) as f32 / repeats as f32;
        // A complete source drawing owns its entire plot, but its physical
        // prop must not fill that plot merely because the bitmap is16px wide.
        if matches!(p.kind,ModelKind::GameConsole|ModelKind::Stool|ModelKind::ArcadeStool
            |ModelKind::Chair|ModelKind::LinkSeat) {
            let center=(left+right)*0.5;
            let pixels=if p.kind==ModelKind::GameConsole {3.} else {4.};
            let half=(pixels*geometry.tile_width/8.+inset).min((right-left)*0.5);
            left=center-half;right=center+half;
        }
        let first = mesh.solid.positions.len();
        let texture_first=mesh.textured.positions.len();
        if native_gift_shop::traditional_table(map,p,cells,geometry) {
            assert!(selected.append_native_traditional_table(&mut mesh.textured,
                [left+inset,right-inset,north,south],p.height_pixels*geometry.tile_height/8.,
                native_source_swatches(p,geometry,[[0,24],[9,8],[1,24]])));
        } else if native_small_display_source(map,p,cells,geometry) {
            let scale=geometry.tile_height/8.;let cx=(left+right)*0.5;
            let cz=geometry.origin_z+(p.row+1)as f32*geometry.tile_height;
            let hx=6.*geometry.tile_width/8.;let hz=6.*scale;
            assert!(selected.append_native_low_table(&mut mesh.textured,[cx-hx,cx+hx,cz-hz,cz+hz],6.*scale,
                native_source_swatches(p,geometry,[[0,8],[2,9],[2,8]])));
            let flower=crate::interior_models::selected_model(ModelKind::FlowerStand).expect("selected display flower");
            let hx=5.*geometry.tile_width/8.;let hz=5.*scale;
            assert!(flower.append_native_display_flower(&mut mesh.textured,[cx-hx,cx+hx,cz-hz,cz+hz],6.*scale,8.*scale,
                native_source_swatches(p,geometry,[[2,2],[11,2],[11,3],[2,2],[11,2],[2,2],
                    [2,9],[2,8],[0,8],[0,8],[2,8],[2,2]])));
        } else if native_pokecom_equipment::computer(map,p,cells,geometry) {
            assert!(selected.append_native_computer(&mut mesh.textured,
                [left+inset,right-inset,north,south],p.base_pixels*geometry.tile_height/8.,
                p.height_pixels*geometry.tile_height/8.,
                native_source_swatches(p,geometry,native_pokecom_equipment::PAINT)));
        } else if native_radio_studio::handled(map,p,cells,geometry) {
            native_radio_studio::append(&mut mesh.textured,map,p,cells,geometry,[left+inset,right-inset,north,south],selected,tables);
        } else if native_radio_studio::computer(map,p,cells,geometry) {
            native_radio_studio::append_computer(&mut mesh.textured,p,cells,geometry,[left+inset,right-inset,north,south],selected);
        } else if native_radio_studio::equipment_size(map,p,cells,geometry).is_some() {
            native_radio_studio::append_equipment(&mut mesh.textured,p,geometry,[left+inset,right-inset,north,south],selected);
        } else if native_radio_room::stool_source(map,p,cells,geometry) {
            let paint=native_source_swatches(p,geometry,[[8,5],[3,9]]);
            selected.append_native_stool(&mut mesh.textured,[left+inset,right-inset,north,south],
                p.height_pixels*geometry.tile_height/8.,paint[0],paint[1]);
        } else if native_radio_room::matches(map,p,cells,geometry) {
            native_radio_room::append(&mut mesh.textured,geometry,p,cells,
                [left+inset,right-inset,north,south],selected,world_origin);
        } else if native_house_plant::source(map,p,cells,geometry) {
            native_house_plant::append(&mut mesh.textured,geometry,p,[left+inset,right-inset,north,south],selected);
        } else if native_port_plant(map,p,cells,geometry) {
            assert!(selected.append_native_safari_plant(&mut mesh.textured,
                [left+inset,right-inset,north,south],p.height_pixels*geometry.tile_height/8.,
                native_source_swatches(p,geometry,[[3,3],[4,4],[0,0],[4,19],[4,20],[3,19]])));
        } else if native_gate_plant_source(map,p,cells,geometry) {
            let pot_y=(p.height-1)*8;
            let swatches=native_source_swatches(p,geometry,
                [[3,1],[6,1],[3,1],[3,pot_y+1],[1,pot_y],[3,pot_y]]);
            if p.kind==ModelKind::PottedPlant {
                // The native two-cell drawing is a compact leafy crown over a pot.
                assert!(selected.append_native_facility_plant(&mut mesh.textured,
                    [left+inset,right-inset,north,south],16.*geometry.tile_height/8.,swatches));
            } else {
                assert!(selected.append_native_gate_planter(&mut mesh.textured,
                    [left+inset,right-inset,north,south],p.height_pixels*geometry.tile_height/8.,
                    [swatches[0],swatches[1],swatches[1],swatches[2],swatches[3],swatches[4],swatches[5]]));
            }
        } else if native_mansion_plant(map,p,cells,geometry) {
            assert!(selected.append_native_facility_plant(&mut mesh.textured,
                [left+inset,right-inset,north,south],16.*geometry.tile_height/8.,
                native_source_swatches(p,geometry,MANSION_PLANT_PIXELS)));
        } else if native_facility_plant(map,p,cells,geometry) {
            assert!(selected.append_native_facility_plant(&mut mesh.textured,
                [left+inset,right-inset,north,south],p.height_pixels*geometry.tile_height/8.,
                native_source_swatches(p,geometry,[[2,2],[2,7],[6,4],[3,25],[4,25],[6,16]])));
        } else if native_classroom::matches(map,p,cells,geometry) {
            native_classroom::append(&mut mesh.textured,geometry,p,[left,right,north,south],selected);
        } else if native_safari_office::matches(map,p,cells,geometry) {
            native_safari_office::append(&mut mesh.textured,geometry,p,
                [left+inset,right-inset,north,south]);
        } else if traditional_wood_wall::source(map,p,cells,geometry) {
            traditional_wood_wall::append(&mut mesh.textured,geometry,p);
        } else if native_gift_shop::traditional_cushion(map,p,cells,geometry) {
            native_gift_shop::append(&mut mesh.textured,geometry,p,ModelKind::Cushion,selected,
                [left+inset,right-inset,north,south]);
        } else if let Some(paint)=native_mansion_clock_source(map,p,cells,geometry) {
            assert!(selected.append_native_gift_fixture(&mut mesh.textured,
                [left+inset,right-inset,north,south],p.base_pixels*geometry.tile_height/8.,
                p.height_pixels*geometry.tile_height/8.,paint));
        } else if let Some(kind)=native_gift_shop::source(map,p,cells,geometry) {
            native_gift_shop::append(&mut mesh.textured,geometry,p,kind,selected,
                [left+inset,right-inset,north,south]);
        } else if native_clinic_fixtures::station(map,p,cells,geometry) {
            native_clinic_fixtures::append_station(&mut mesh.textured,geometry,p,selected);
        } else if let Some(down)=native_clinic_fixtures::stair(map,p,cells,geometry) {
            native_clinic_fixtures::append_stair(&mut mesh.textured,geometry,p,
                [left,right,north,south],down,selected);
        } else if native_clinic::healing_source(map,p,cells,geometry) {
            native_clinic_fixtures::append_backing(&mut mesh.textured,map,p,cells,geometry,tables);
            native_clinic::append_healing(&mut mesh.textured,geometry,p,[left+inset,right-inset,north,south],
                p.height_pixels*geometry.tile_height/8.);
        } else if native_clinic::wall_source(map,p,cells,geometry) {
            native_clinic::append_wall(&mut mesh.textured,geometry,p,[left,right,north,south],
                p.height_pixels*geometry.tile_height/8.);
        } else if native_clinic::cabinet_source(map,p,cells,geometry) {
            native_clinic_fixtures::append_backing(&mut mesh.textured,map,p,cells,geometry,tables);
            native_clinic::append_cabinet(&mut mesh.textured,geometry,p,
                [left+inset,right-inset,south-6.*geometry.tile_height/8.,south],
                16.*geometry.tile_height/8.);
        } else if native_clinic::pc_source(map,p,cells,geometry) || native_clinic::console_source(map,p,cells,geometry) {
            native_clinic_fixtures::append_backing(&mut mesh.textured,map,p,cells,geometry,tables);
            native_clinic::append_terminal(&mut mesh.textured,geometry,p,[left+inset,right-inset,north,south],
                p.height_pixels*geometry.tile_height/8.);
        } else if let Some(swatches)=native_elm_central_source(map,p,cells,geometry) {
            append_native_elm_desk(&mut mesh.textured,geometry,p,[left,right,north,south],swatches);
        } else if let Some(swatches)=native_elm_fixture_source(map,p,cells,geometry) {
            append_native_elm_healing_machine(&mut mesh.textured,geometry,p,[left,right,north,south],swatches);
        } else if let Some((swatches,desk))=native_lab_workstation_source(map,p,cells,geometry) {
            assert!(selected.append_native_lab_workstation(&mut mesh.textured,[left+inset,right-inset,north,south],
                p.height_pixels*geometry.tile_height/8.,swatches,desk));
        } else if let Some(swatches)=native_mansion_workstation_source(map,p,cells,geometry) {
            assert!(selected.append_native_workstation(&mut mesh.textured,[left+inset,right-inset,north,south],
                p.height_pixels*geometry.tile_height/8.,swatches));
            let seat=Placement{kind:ModelKind::LinkSeat,row:p.row+2,width:2,height:2,..p.clone()};
            let swatches=native_link_seat_source(map,&seat,cells,geometry).expect("complete office chair drawing");
            let center_x=left+geometry.tile_width;
            let center_z=geometry.origin_z+(p.row+3)as f32*geometry.tile_height;
            let half_x=geometry.tile_width*0.5;let half_z=geometry.tile_height*0.5;
            let seat_first=mesh.textured.positions.len();
            assert!(crate::interior_models::selected_model(ModelKind::LinkSeat).expect("selected office seat").append_native_link_seat(&mut mesh.textured,
                [center_x-half_x,center_x+half_x,center_z-half_z,center_z+half_z],
                11.*geometry.tile_height/8.,swatches));
            // This chair sits south of its own desk, facing the desktop.
            turn_chair_half(&mut mesh.textured,seat_first,[center_x,center_z]);
        } else if let Some(swatches)=native_link_seat_source(map,p,cells,geometry) {
            assert!(selected.append_native_link_seat(&mut mesh.textured,[left+inset,right-inset,north,south],
                p.height_pixels*geometry.tile_height/8.,swatches));
        } else if let Some(swatches)=native_arcade_source(map,p,cells,geometry) {
            assert!(selected.append_native_arcade(&mut mesh.textured,[left+inset,right-inset,north,south],
                p.height_pixels*geometry.tile_height/8.,swatches));
        } else if let Some(swatches)=native_center_counter_source(map,p,cells,geometry) {
            assert!(selected.append_native_store_counter(&mut mesh.textured,[left,right,north,south],
                p.height_pixels*geometry.tile_height/8.,swatches,false));
            // The circles are source-authored countertop markings. Preserve
            // their live drawing above the canonical cap rather than adding
            // unrelated objects or replacing them with a solid swatch.
            for x in 0..p.width {
                if cells[p.row*geometry.width+p.column+x].source.tile_index!=0x0c {continue;}
                let w=left+(right-left)*x as f32/p.width as f32;
                let e=left+(right-left)*(x+1)as f32/p.width as f32;
                let h=(p.height_pixels+0.002)*geometry.tile_height/8.;
                let uv=geometry.uv(p.column+x,p.row);
                append_quad(&mut mesh.textured,[[w,h,north],[w,h,south],[e,h,south],[e,h,north]],
                    [0.,1.,0.],[[uv.0,uv.2],[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2]],[1.;4]);
            }
        } else if let Some(swatches)=native_mart_counter_source(map,p,cells,geometry) {
            assert!(selected.append_native_store_counter(&mut mesh.textured,[left,right,north,south],
                p.height_pixels*geometry.tile_height/8.,swatches,false));
        } else if native_station_ticket_source(map,p,cells,geometry) {
            append_native_station_ticket_boxes(&mut mesh.textured,geometry,p,selected,[left,right,north,south]);
        } else if native_store_vending_source(map,p,cells,geometry) {
            let first=mesh.textured.positions.len();
            let swatches=native_source_swatches(p,geometry,[[1,1],[4,10],[0,8],[2,1],[2,8]]);
            assert!(selected.append_native_beverage_machine(&mut mesh.textured,[left,right,north,south],
                p.height_pixels*geometry.tile_height/8.,swatches));
            append_store_vending_roof(&mut mesh.textured,first,
                p.height_pixels*geometry.tile_height/8.,geometry.tile_height/8.,swatches);
        } else if native_station_wall_source(map,p,cells,geometry) {
            append_native_station_wall(&mut mesh.textured,geometry,p,[left,right,north,south],piece_source.metatile_id==0x0a);
        } else if p.kind==ModelKind::WallPanel && matches!(p.width,2|4) && p.height==2
            && p.indices(geometry.width).enumerate().all(|(j,i)|
                crate::train_station::railing_local(map,&cells[i].source)==Some(((j%p.width)as u8,(j/p.width)as u8,p.width,2))) {
            append_native_station_railing(&mut mesh.textured,geometry,p,[left,right,north,south]);
        } else if native_wise_divider(map,p,cells,geometry) {
            append_wise_divider(&mut mesh.textured,geometry,p,cells,[left,right,north,south]);
        } else if p.kind == ModelKind::WallTraditional
            && cells[p.row * geometry.width + p.column].source.tileset_id.as_ref() == "tower" {
            let source = &cells[p.row * geometry.width + p.column].source;
            let source_uv = |pixel: [usize; 2]| {
                let pixel = if source.tile_index == 0x41 {
                    // Row2's white dash is a tiny source glint, not paint for
                    // the whole timber frame. Keep framing on native gold170.
                    if pixel==[0,2] {[1,3]}else{pixel}
                } else {
                    match pixel {
                        [0, 0] => [0, 0], [0, 2] => if source.tile_index==0x11 {[2,3]}else{[1,3]},
                        [0, 1] => [0, 1], [4, 0] => [2, 0],
                        _ => unreachable!("validated timber materials"),
                    }
                };
                let uv = geometry.uv(p.column, p.row);
                [uv.0 + (uv.1 - uv.0) * (pixel[0] as f32 + 0.5) / 8.,
                 uv.2 + (uv.3 - uv.2) * (pixel[1] as f32 + 0.5) / 8.]
            };
            if source.tile_index == 0x41 {
                let facing = if source.subtile_column < 2 { 1.0 } else { -1.0 };
                append_side_wall(&mut mesh.textured,
                    [left, right, north, south], p.height_pixels * geometry.tile_height / 8.0,
                    facing, geometry.tile_width, source_uv);
            } else {
                crate::dungeon_models::model(crate::dungeon_models::Kind::TimberWall).append_tower_timber(
                    &mut mesh.textured, guardian_wall_bounds(map,p,cells,geometry,[left, right, north, south]),
                    p.height_pixels * geometry.tile_height / 8.0, source_uv);
            }
        } else if native_gate_wall || native_gate_doors {
            append_native_gate_wall(&mut mesh.textured,geometry,p,[left,right,north,south]);
        } else if p.kind==ModelKind::TrashCan {
            let uv=geometry.uv(p.column,p.row);
            let swatches=[[6.,1.],[2.,2.],[1.,1.]].map(|v|[uv.0+(uv.1-uv.0)*(v[0]+0.5)/8.,uv.2+(uv.3-uv.2)*(v[1]+0.5)/8.]);
            let center=(left+right)*0.5;let half=5.*geometry.tile_width/8.;
            assert!(selected.append_native_trashcan(&mut mesh.textured,[center-half,center+half,north,south],8.*geometry.tile_height/8.,swatches),"native shared trashcan requires named source materials");
        } else if native_cafe_wall {
            assert!(p.height<=2 && p.base_pixels>=0.,"native cafe wall requires source course bounds");
            append_kanto_house_wall(&mut mesh.textured,geometry,p,[left,right,north,south]);
        } else if reviewed_native_room(map) && native_house_window_source(p,cells,geometry) {
            append_native_house_window(&mut mesh.textured,geometry,p,cells,[left,right,north,south]);
        } else if reviewed_native_room(map) && p.kind==ModelKind::WallDomestic
            && (p.height==2 || native_player_wall) && p.indices(geometry.width).all(|i| {
                let s=&cells[i].source;
                matches!((s.tileset_id.as_ref(),s.tile_index),
                    ("house",0) | ("players_house",0x11|0x32|0x33|0x34) | ("players_room",0x02))
            }) {
            append_kanto_house_wall(&mut mesh.textured,geometry,p,[left,right,north,south]);
        } else if native_pokecom_seats::matches(map,p,cells,geometry)
            || kanto_native_floor_map(map) && map.ends_with("Pokecenter1F")
            && p.kind==ModelKind::LinkSeat && p.width==2 && p.height==2
            && p.indices(geometry.width).enumerate().all(|(j,i)| {
                let source=&cells[i].source;
                source.tileset_id.as_ref()=="pokecenter"
                    && matches!(source.metatile_id,0x2e|0x2f)
                    && source.tile_index==[0x48,0x49,0x58,0x59][j]
            }) {
            append_native_center_lounge_seat(&mut mesh.textured,geometry,p,
                [left+inset,right-inset,north,south],
                if native_pokecom_seats::matches(map,p,cells,geometry) {[2,12]}else{[7,12]});
        } else if map=="CeladonCafe" && p.kind==ModelKind::ArcadeStool && p.width==2 && p.height==2
            && p.indices(geometry.width).enumerate().all(|(j,i)| {
                let s=&cells[i].source;
                s.tileset_id.as_ref()=="game_corner" && matches!(s.metatile_id,5|6|0x19)
                    && s.subtile_column==piece_source.subtile_column+j as u8%2
                    && s.subtile_row==piece_source.subtile_row+j as u8/2
                    && s.tile_index==[0x0a,0x0b,0x1a,0x1b][j]
            }) {
            append_native_rectangular_stool(&mut mesh.textured,geometry,p,
                [left+inset,right-inset,north,south]);
        } else if (reviewed_native_room(map) || map=="CeladonCafe")
            && p.kind==ModelKind::Stool && p.width==2 && p.height==2
            && matches!(piece_source.tileset_id.as_ref(),"house"|"players_house")
            && p.indices(geometry.width).enumerate().all(|(j,i)| {
                crate::house::furniture_local(&cells[i].source)
                    ==Some(((j%2) as u8,(j/2) as u8,crate::house::FurnitureKind::Stool))
            }) {
            if piece_source.tileset_id.as_ref()=="players_house" {
                append_native_round_wood_stool(&mut mesh.textured,geometry,p,
                    [left+inset,right-inset,north,south]);
            } else {
                append_native_rectangular_stool(&mut mesh.textured,geometry,p,
                    [left+inset,right-inset,north,south]);
            }
        } else if map=="SoulHouse" && p.kind==ModelKind::Memorial
            && p.width==2 && p.height==2 && p.indices(geometry.width).enumerate().all(|(j,i)| {
                crate::house::soul_house_bench_local(&cells[i].source)
                    ==Some(((j%2) as u8,(j/2) as u8))
            }) {
            // All three swatches belong to this complete memorial's upper-left
            // tile37. Its gold body, brown recess and pale trim remain distinct.
            let sample=|pixel:[usize;2]| {
                let uv=geometry.uv(p.column+pixel[0]/8,p.row+pixel[1]/8);
                [uv.0+(uv.1-uv.0)*((pixel[0]%8) as f32+0.5)/8.,
                 uv.2+(uv.3-uv.2)*((pixel[1]%8) as f32+0.5)/8.]
            };
            assert!(selected.append_native_memorial(&mut mesh.textured,
                [left+inset,right-inset,north,south],
                p.base_pixels*geometry.tile_height/8.,p.height_pixels*geometry.tile_height/8.,
                [sample([1,1]),sample([4,3]),sample([4,1])]));
        } else if p.width==2 && p.height==4 && piece_source.tileset_id.as_ref()=="mart"
            && ((p.kind==ModelKind::RetailShelf && piece_source.metatile_id==0x13)
                || (reviewed_retail_room(map) && matches!((p.kind,piece_source.metatile_id,piece_source.tile_index),
                    (ModelKind::RetailShelf,0x12|0x2c,0x26)
                    |(ModelKind::RetailShelf,0x15|0x27,0x0c)
                    |(ModelKind::RetailRefrigerator,0x14,0x0c)))) {
            let cabinet_start=mesh.textured.positions.len();
            let cabinet_indices=mesh.textured.indices.len();
            append_native_retail_display(&mut mesh.textured,geometry,p,
                [left,right,north,south],piece_source.tile_index==0x26);
            if native_store_aisle_shelf_source(map,p,cells,geometry) {
                let joins=native_store_shelf_neighbors(map,p,cells,geometry);
                let faces=mesh.textured.indices.split_off(cabinet_indices);
                mesh.textured.indices.extend(faces.chunks_exact(3).filter(|tri| {
                    ![(joins[0],left,-1.),(joins[1],right,1.)].iter().any(|&(joined,x,normal)| {
                        joined&&tri.iter().all(|&i|mesh.textured.positions[i as usize][0]==x
                            &&mesh.textured.normals[i as usize][0]==normal)
                    })
                }).flatten().copied());
                let range=cabinet_start..mesh.textured.positions.len();
                mesh.textured.cutaway_ranges.push(range.clone());
                mesh.depth_coverage_textured_ranges.push(range);
            }
        } else if native_store_plant_source(map,p,cells,geometry) {
            assert!(selected.append_native_facility_plant(&mut mesh.textured,
                [left+inset,right-inset,north,south],p.height_pixels*geometry.tile_height/8.,
                // a7 uses bank one: renderer resolves it to PNG tile87.
                // Its row2 is coral red; row1 is the pale pink pot rim.
                native_source_swatches(p,geometry,[[3,2],[3,4],[3,1],[3,26],[2,25],[0,24]])));
        } else if p.kind == ModelKind::Cushion
            && cells[p.row * geometry.width + p.column].source.tileset_id.as_ref()=="mart" {
            if native_store_seat_source(map,p,cells,geometry) {
                append_native_store_seat(&mut mesh.textured,[left+inset,right-inset,north,south],
                    geometry.tile_height/8.,native_source_swatches(p,geometry,[[1,1],[3,3],[0,1]]));
            } else {
                append_store_seat(&mut mesh.solid,[left+inset,right-inset,north,south],geometry.tile_height/8.0);
            }
        } else if p.kind == ModelKind::DiningTable && p.width == 4 && p.height == 3
            && piece_source.tileset_id.as_ref() == "game_corner" && piece_source.metatile_id == 0x2f {
            append_arcade_game_table(&mut mesh.textured, geometry, p, [left,right,north,south]);
        } else if native_station_chair_source(map,p,cells,geometry)||native_radio_room::chair_source(map,p,cells,geometry) {
            let uv=geometry.uv(p.column,p.row);
            let sample=|x:f32,y:f32|[uv.0+(uv.1-uv.0)*(x+0.5)/8.,uv.2+(uv.3-uv.2)*(y+0.5)/8.];
            assert!(selected.append_native_station_chair(&mut mesh.textured,
                [left+inset,right-inset,north,south],p.height_pixels*geometry.tile_height/8.,
                [sample(4.,2.),if native_radio_room::chair_source(map,p,cells,geometry) {sample(4.,2.)}else{sample(4.,7.)}]));
        } else if native_station_plant_source(map,p,cells,geometry) {
            let sample=|pixel:[usize;2]| {
                let uv=geometry.uv(p.column+pixel[0]/8,p.row+pixel[1]/8);
                [uv.0+(uv.1-uv.0)*((pixel[0]%8)as f32+0.5)/8.,
                 uv.2+(uv.3-uv.2)*((pixel[1]%8)as f32+0.5)/8.]
            };
            let bottom=(p.height-1)*8;
            assert!(selected.append_native_station_plant(&mut mesh.textured,
                [left+inset,right-inset,north,south],p.height_pixels*geometry.tile_height/8.,
                [[10,3],[9,3],[10,3],[14,3],[3,bottom+2],[3,bottom+1],[4,0]].map(sample)));
            assert!(model(ModelKind::FlowerStand).append_native_station_blooms(&mut mesh.textured,
                [left+inset,right-inset,north,south],22.*geometry.tile_height/8.,
                [[2,4],[3,4],[2,1]].map(sample)));
        } else if map=="CeladonCafe" && p.kind==ModelKind::CafeTable {
            let sample=|pixel:[usize;2]| {
                let uv=geometry.uv(p.column+pixel[0]/8,p.row+pixel[1]/8);
                [uv.0+(uv.1-uv.0)*((pixel[0]%8) as f32+0.5)/8.,
                 uv.2+(uv.3-uv.2)*((pixel[1]%8) as f32+0.5)/8.]
            };
            let scale=geometry.tile_height/8.;
            assert!(selected.append_native_cafe_table(&mut mesh.textured,
                [left+inset,right-inset,north,south],16.*scale,
                [sample([4,14]),sample([12,34]),sample([14,30])]));
            append_cafe_basket(&mut mesh.textured,[(left+right)*0.5,(north+south)*0.5],
                scale,[sample([11,13]),sample([13,10]),sample([15,12])]);
        } else if reviewed_native_room(map)
            && let Some(swatches)=native_house_television_source(p,cells,geometry) {
            let sample=|pixel:[usize;2]| {
                let uv=geometry.uv(p.column+pixel[0]/8,p.row+pixel[1]/8);
                [uv.0+(uv.1-uv.0)*((pixel[0]%8)as f32+0.5)/8.,
                 uv.2+(uv.3-uv.2)*((pixel[1]%8)as f32+0.5)/8.]
            };
            let bounds=[left+inset,right-inset,north,south];
            let height=p.height_pixels*geometry.tile_height/8.;
            let appended=if p.height==2 {
                selected.append_native_television_screen(&mut mesh.textured,bounds,height,swatches.map(sample))
            } else {selected.append_native_television(&mut mesh.textured,bounds,height,swatches.map(sample))};
            assert!(appended,"native television requires cabinet and CRT material roles");
        } else if native_house_dining_table(map,cells,geometry,p) {
            let table_first=mesh.textured.positions.len();
            let table_index=mesh.textured.indices.len();
            // Verified table tile26: medium-gold timber and darker gold cloth.
            // Use the complete object's palette, retaining its selected geometry.
            let uv=geometry.uv(p.column,p.row);
            let sample=|pixel:[usize;2]|[uv.0+(uv.1-uv.0)*(pixel[0] as f32+0.5)/8.,
                uv.2+(uv.3-uv.2)*(pixel[1] as f32+0.5)/8.];
            assert!(selected.append_native_dining_table(&mut mesh.textured,
                [left+inset,right-inset,north,south],p.height_pixels*geometry.tile_height/8.,
                if native_mansion_lounge_table(map,p,cells,geometry) {
                    // Verified tile47 pixel(4,4), palette5: plain gold timber.
                    [native_source_swatches(p,geometry,[[12,12]])[0];2]
                } else if native_oak_table(map,p,cells,geometry) {
                    // The central tile06 is the uninterrupted gold worktop.
                    [native_source_swatches(p,geometry,[[10,4]])[0];2]
                } else if p.width==8 {
                    [sample([17,9]);2]
                } else if p.height==3 {
                    [sample([10,4]);2]
                } else if piece_source.tileset_id.as_ref()=="players_house" {
                    [sample([10,12]);2]
                } else {[sample([7,2]),sample([5,2])]}),"native complete square dining table requires named wood and cloth roles");
            // Each accepted house/bedroom/lab/mansion drawing ends with its
            // perspective apron. Paint only the remaining source top rows.
            native_tabletop::append(&mut mesh.textured,table_index,geometry,p,
                [left+inset,right-inset,north,south],native_large_house_table(map,p,cells,geometry));
            if native_large_house_table(map,p,cells,geometry) {
                native_table_centerpiece::append(&mut mesh.textured,geometry,p,
                    [left+inset,right-inset,north,south]);
            }
            // Reveal only this complete table (including its source centerpiece),
            // retaining opaque floor backing and the original physical dimensions.
            let table=table_first..mesh.textured.positions.len();
            if native_large_house_table(map,p,cells,geometry) {
                mesh.layered_alpha_textured_ranges.push(table.clone());
            }
            // A table has few overlapping surfaces. Use the existing smooth
            // sorted transparency path instead of cabinet coverage dithering.
            mesh.textured.cutaway_ranges.push(table);
        } else if p.kind == ModelKind::ArcadeStool {
            let cx = (left + right) * 0.5;
            let cz = geometry.origin_z + (p.row as f32 + p.height as f32 * 0.5) * geometry.tile_height;
            let radius_x = 4.0 * geometry.tile_width / 8.0;
            let radius_z = 4.0 * geometry.tile_height / 8.0;
            selected.append_fitted(&mut mesh.solid,
                [cx - radius_x, cx + radius_x, cz - radius_z, cz + radius_z],
                p.base_pixels * geometry.tile_height / 8.0,
                p.height_pixels * geometry.tile_height / 8.0);
        } else if p.kind == ModelKind::ArcadePair {
            append_outward_arcade_pair(&mut mesh.solid,
                [left + inset, right - inset, north, south],
                p.base_pixels * geometry.tile_height / 8.0,
                p.height_pixels * geometry.tile_height / 8.0);
        } else if arcade_poster {
            // A sign is attached to a closed wall, not suspended over a hole.
            // Its exact casino drawing owns the backing without a new claim.
            let wall_south = south - poster_relief;
            model(ModelKind::ArcadeWall).append_fitted(&mut mesh.solid,
                [left, right, wall_south - 3.0 * geometry.tile_height / 8.0, wall_south],
                0.0, 28.0 * geometry.tile_height / 8.0);
            model(ModelKind::ArcadeWall).apply_arcade_wall_finish(&mut mesh.solid, first);
            selected.append_fitted(&mut mesh.solid,
                [left + inset, right - inset, north, south],
                p.base_pixels * geometry.tile_height / 8.0,
                p.height_pixels * geometry.tile_height / 8.0);
        } else if p.kind == ModelKind::ArcadeMachine
            && cells[p.row * geometry.width + p.column].source.tileset_id.as_ref() == "game_corner"
            && matches!(cells[p.row * geometry.width + p.column].source.metatile_id, 0x07 | 0x0b) {
            let source = &cells[p.row * geometry.width + p.column].source;
            append_side_arcade(&mut mesh.solid, [left + inset, right - inset, north, south],
                p.base_pixels * geometry.tile_height / 8.0, 21.0 * geometry.tile_height / 8.0,
                if source.subtile_column < 2 { -1.0 } else { 1.0 });
        } else if reviewed_native_room(map) && native_red_bed_source(p,cells,geometry) {
            let sample=|pixel:[usize;2]| {
                let uv=geometry.uv(p.column+pixel[0]/8,p.row+pixel[1]/8);
                [uv.0+(uv.1-uv.0)*((pixel[0]%8)as f32+0.5)/8.,
                 uv.2+(uv.3-uv.2)*((pixel[1]%8)as f32+0.5)/8.]
            };
            assert!(selected.append_native_bed(&mut mesh.textured,[left+inset,right-inset,north,south],
                p.base_pixels*geometry.tile_height/8.,p.height_pixels*geometry.tile_height/8.,
                [[2,4],[1,4],[3,19],[3,10]].map(sample)),
                "native Red bed requires named frame, hardware, cover and linen roles");
        } else if reviewed_native_room(map) && native_house_radio_source(p,cells,geometry) {
            let scale=geometry.tile_height/8.;
            let b=[left+inset,right-inset,north,south];
            let cabinet=crate::interior_models::selected_model(ModelKind::DrawerCabinet).unwrap();
            // The lower source row is a gold cabinet, below the blue/gray radio.
            assert!(cabinet.append_native_home_fixture(&mut mesh.textured,b,0.,8.*scale,
                native_source_swatches(p,geometry,[[1,16],[4,19],[4,18],[4,19]])));
            assert!(selected.append_native_house_radio(&mut mesh.textured,b,8.*scale,8.*scale,
                native_source_swatches(p,geometry,[[2,13],[0,0],[4,4],[0,8],[4,12]])));
        } else if reviewed_native_room(map)
            && let Some(swatches) = native_home_fixture_source(p, cells, geometry) {
            let sample = |pixel: [usize; 2]| {
                let uv = geometry.uv(p.column + pixel[0] / 8, p.row + pixel[1] / 8);
                [uv.0 + (uv.1 - uv.0) * ((pixel[0] % 8) as f32 + 0.5) / 8.,
                 uv.2 + (uv.3 - uv.2) * ((pixel[1] % 8) as f32 + 0.5) / 8.]
            };
            assert!(selected.append_native_home_fixture(&mut mesh.textured,
                [left + inset, right - inset, north, south],
                p.base_pixels * geometry.tile_height / 8.,
                p.height_pixels * geometry.tile_height / 8., swatches.map(sample)),
                "native home fixture requires all canonical material roles");
        } else if reviewed_native_room(map)
            && let Some(swatches)=native_computer_source(p,cells,geometry) {
            let sample=|pixel:[usize;2]| {
                let uv=geometry.uv(p.column+pixel[0]/8,p.row+pixel[1]/8);
                [uv.0+(uv.1-uv.0)*(pixel[0]%8)as f32/8.+(uv.1-uv.0)/16.,
                 uv.2+(uv.3-uv.2)*(pixel[1]%8)as f32/8.+(uv.3-uv.2)/16.]
            };
            let b=[left+inset,right-inset,north,south];
            let base=p.base_pixels*geometry.tile_height/8.;let height=p.height_pixels*geometry.tile_height/8.;
            let appended=if piece_source.tileset_id.as_ref()=="house" {
                selected.append_native_cabinet_computer(&mut mesh.textured,b,base,height,swatches.map(sample),
                    crate::interior_models::selected_model(ModelKind::DrawerCabinet).unwrap(),
                    native_source_swatches(p,geometry,[[1,16],[4,20],[3,17],[4,20]]))
            } else {selected.append_native_computer(&mut mesh.textured,b,base,height,swatches.map(sample))};
            assert!(appended,"designated native computer source requires complete model material roles");
        } else if let Some(book_pixels)=native_bookcase_source(map,p,cells,geometry) {
            let bookcase_first=mesh.textured.positions.len();
            let bookcase_indices=mesh.textured.indices.len();
            // Swatches belong to this repeated cabinet's top and front, not
            // to a global guessed wood palette or a neighboring floor tile.
            let uv_at = |pixel: [usize;2]| {
                let uv = geometry.uv(p.column + unit*2 + pixel[0]/8,
                    p.row + pixel[1]/8);
                [uv.0+(uv.1-uv.0)*(pixel[0]%8) as f32/8.+(uv.1-uv.0)/16.,
                 uv.2+(uv.3-uv.2)*(pixel[1]%8) as f32/8.+(uv.3-uv.2)/16.]
            };
            let light = match piece_source.tileset_id.as_ref() {"players_house"=>[1,8],"players_room"=>[1,7],_=>[3,3]};
            assert!(selected.append_native_bookcase_with_books(&mut mesh.textured,
                [left+inset,right-inset,north,south],
                p.base_pixels*geometry.tile_height/8.,p.height_pixels*geometry.tile_height/8.,
                [uv_at([0,8]),uv_at([2,8]),uv_at(light)],book_pixels.map(uv_at)),
                "native cabinet and books require authenticated material ownership");
            append_native_bookcase_base(&mut mesh.textured,geometry,p,cells,unit,
                bookcase_first,bookcase_indices,uv_at([2,8]));
        } else if native_radio_stair::stair(map,p,cells,geometry).is_some() {
            native_radio_stair::append(mesh,map,p,cells,geometry,[left,right,north,south],selected);
        } else if native_mansion_architecture::stair(map,p,cells,geometry).is_some() {
            native_mansion_architecture::append_stair(&mut mesh.textured,map,p,cells,geometry,
                [left,right,north,south],selected);
        } else if native_mansion_architecture::wall(map,p,cells,geometry) {
            native_mansion_architecture::append_wall(mesh,geometry,p,cells,[left,right,north,south]);
        } else if let Some(pixels)=native_home_stair::source(map,p,cells,geometry) {
            assert!(selected.append_native_stair(&mut mesh.textured,
                [left+inset,right-inset,north,south],p.base_pixels*geometry.tile_height/8.,
                p.height_pixels*geometry.tile_height/8.,native_source_swatches(p,geometry,pixels)));
        } else {
            selected.append_fitted(
                &mut mesh.solid,
                [left + inset, right - inset, north, south],
                p.base_pixels * geometry.tile_height / 8.0,
                p.height_pixels * geometry.tile_height / 8.0,
            );
        }
        if p.kind == ModelKind::ArcadeWall && !native_cafe_wall && !native_safari_office::matches(map,p,cells,geometry) {
            selected.apply_arcade_wall_finish(&mut mesh.solid, first);
        } else if matches!(p.kind, ModelKind::PrizeBooth | ModelKind::ArcadePrizeRack) {
            selected.apply_room_finish(&mut mesh.solid, first);
        }
        if p.kind == ModelKind::PottedPlant
            && !native_safari_office::matches(map,p,cells,geometry)
            && !native_store_plant_source(map,p,cells,geometry)
            && matches!(cells[p.row * geometry.width + p.column].source.tileset_id.as_ref(), "mart" | "game_corner") {
            selected.apply_room_palette(&mut mesh.solid, first);
        }
        if cells[p.row * geometry.width + p.column].source.tileset_id.as_ref() == "game_corner" {
            if p.kind == ModelKind::ArcadeMachine && native_arcade_source(map,p,cells,geometry).is_none()
                && !matches!(cells[p.row * geometry.width + p.column].source.metatile_id, 0x07 | 0x0b) {
                selected.apply_arcade_finish(&mut mesh.solid, first);
            } else if p.kind == ModelKind::ReceptionCounter && !native_safari_office::matches(map,p,cells,geometry) {
                selected.apply_arcade_finish(&mut mesh.solid, first);
            }
        }
        if cells[p.row * geometry.width + p.column].source.tileset_id.as_ref() == "gate" {
            if p.kind == ModelKind::Chair {
                let block = cells[p.row * geometry.width + p.column].source.metatile_id;
                let facing = match block { 0x32 => -1.0, 0x35 => 1.0, _ => 0.0 };
                turn_counter_chair(&mut mesh.solid, first,
                    [(left + right) * 0.5, (north + south) * 0.5],
                    [(right - left) * 0.5 - inset, (south - north) * 0.5], facing);
            }
            vary_gate_furnishing(&mut mesh.solid, first, p.kind,
                [world_origin[0] + p.column as i32, world_origin[1] + p.row as i32],
                [(left + right) * 0.5, (north + south) * 0.5]);
        }
        if let Some(facing)=chair_table_facing(p,tables,chair_source_phase(p,piece_source)) {
            let old=if p.kind==ModelKind::LinkSeat&&native_classroom::matches(map,p,cells,geometry) {2}
            else if piece_source.tileset_id.as_ref()=="gate" {
                match piece_source.metatile_id {0x32=>3,0x35=>1,_=>0}
            }else{0};
            let turns=(facing+4-old)%4;
            let center=[(left+right)*0.5,(north+south)*0.5];
            let half=[(right-left)*0.5-inset,(south-north)*0.5];
            for (surface,start) in [(&mut mesh.solid,first),(&mut mesh.textured,texture_first)] {
                if turns==2 {
                    turn_chair_half(surface,start,center);
                }else if turns!=0 {
                    turn_counter_chair(surface,start,center,half,if turns==1 {1.}else{-1.});
                }
            }
        }
    }
    if p.kind == ModelKind::PrizeBooth { append_prize_booth_art(mesh, cells, geometry, p); }
    if p.kind == ModelKind::CarpetCloth {
        append_live_carpet_art(mesh, geometry, p);
    }
    if p.kind == ModelKind::PictureFrame {
        append_framed_source_art(mesh, geometry, p, [west, east, north, south], repeats);
        if reviewed_native_room(map)&&matches!(piece_source.tileset_id.as_ref(),"house"|"mansion") {
            // Mount the frame on the connected wall instead of embedding its
            // recessed artwork inside the newly restored backing volume.
            let relief=p.depth_pixels*geometry.tile_height/8.;
            for v in &mut mesh.solid.positions[object_solid_first..] {v[2]+=relief;}
            for v in &mut mesh.textured.positions[corridor_furniture_first..] {v[2]+=relief;}
        }
    }
    if native_mansion_corridor_furniture(map,p,cells,geometry) {
        // Own the complete furniture assembly, including books/clock face and
        // workstation chair. The source floor was emitted before this range.
        let range=corridor_furniture_first..mesh.textured.positions.len();
        mesh.textured.cutaway_ranges.push(range.clone());
        mesh.depth_coverage_textured_ranges.push(range);
    }
    if native_wise_divider(map,p,cells,geometry) {
        let range=corridor_furniture_first..mesh.textured.positions.len();
        mesh.textured.cutaway_ranges.push(range.clone());
        mesh.layered_alpha_textured_ranges.push(range);
    }
    for i in p.indices(geometry.width) {
        claimed[i] = true;
        if mesh.authored_cells.len() == cells.len() {
            mesh.authored_cells[i] = Some(p.label());
        }
        if let Some(height) = p.footing_pixels {
            if mesh.footing_heights.len() == cells.len() {
                mesh.footing_heights[i] = height * geometry.tile_height / 8.0;
            }
        }
    }
    true
}

/// +Z is the authored chair front. Only aligned, nearby complete table/counter
/// placements establish a relationship; distant or diagonal furniture does not.
pub(super) fn chair_source_phase(seat:&Placement,source:&VisualTileSource)->[usize;2] {
    [(source.subtile_column as usize+4-seat.column%4)%4,
     (source.subtile_row as usize+4-seat.row%4)%4]
}
pub(super) fn chair_table_facing(seat:&Placement,placements:&[Placement],phase:[usize;2])->Option<u8> {
    if !matches!(seat.kind,ModelKind::Chair|ModelKind::LinkSeat) {return None;}
    let rect=|p:&Placement|[p.column as i32,(p.column+p.width)as i32,
        p.row as i32,(p.row+p.height)as i32];
    let [w,e,n,s]=rect(seat);
    placements.iter().filter(|p|matches!(p.kind,ModelKind::DiningTable|ModelKind::LowTable
        |ModelKind::CafeTable|ModelKind::ReceptionCounter|ModelKind::LabCounter
        |ModelKind::Computer|ModelKind::LabWorkstation|ModelKind::LabDisplayTable))
        .filter_map(|p| {
            let [tw,te,tn,ts]=rect(p);
            let (gap,face)=if w<te&&e>tw {
                if s<=tn {(tn-s,0)}else if n>=ts {(n-ts,2)}else{return None;}
            }else if n<ts&&s>tn {
                if e<=tw {(tw-e,1)}else if w>=te {(w-te,3)}else{return None;}
            }else{return None;};
            // Packed classroom rows can put the next desk immediately behind
            // a chair. Prefer its own source block when both edges touch.
            let same_block=(p.column+phase[0])/4==(seat.column+phase[0])/4
                &&(p.row+phase[1])/4==(seat.row+phase[1])/4;
            (gap<=4).then_some((gap,!same_block,face))
        }).min_by_key(|&(gap,other,face)|(gap,other,face)).map(|(_,_,face)|face)
}

fn turn_chair_half(surface:&mut SurfaceMeshData,start:usize,center:[f32;2]) {
    for i in start..surface.positions.len() {
        let v=&mut surface.positions[i];v[0]=2.*center[0]-v[0];v[2]=2.*center[1]-v[2];
        let n=&mut surface.normals[i];
        let old=crate::interior_models::authored_face_color([1.;4],Vec3::from_array(*n));
        n[0]=-n[0];n[2]=-n[2];
        let new=crate::interior_models::authored_face_color([1.;4],Vec3::from_array(*n));
        if let Some(color)=surface.colors.get_mut(i) {
            for c in 0..3 {color[c]*=new[c]/old[c];}
        }
    }
}

include!("cafe_table.rs");

/// Preserve the profile's raised upper/planter fragments and shared 22px crest.
/// Their full source guard is resolved before this native palette path runs.
fn append_native_gate_wall(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,
    b:[f32;4]) {
    let [w,e,n,s]=b;let scale=g.tile_height/8.;
    let crop=if p.height==1{0.}else{2.};
    let base=match p.height{1=>14.,2=>8.,3=>0.,_=>unreachable!()}*scale;
    let top=22.*scale;
    let first=out.positions.len();let indices=out.indices.len();
    architecture_box(out,[w,e,base,top,n,s],[1.;3]);
    let uv=g.uv(p.column,p.row);
    let cap=[uv.0+(uv.1-uv.0)*4.5/8.,uv.2+(uv.3-uv.2)*1.5/8.];
    out.uvs[first..].fill(cap);
    out.indices.drain(indices+6..indices+12);
    for y in 0..p.height {for x in 0..p.width {
        let uv=g.uv(p.column+x,p.row+y);
        let clipped=if y==0{crop}else{0.};
        let high=top-(y as f32*8.-crop).max(0.)*scale;
        let low=top-((y+1) as f32*8.-crop)*scale;
        let left=w+x as f32*g.tile_width;let right=left+g.tile_width;
        let v0=uv.2+(uv.3-uv.2)*clipped/8.;
        append_quad(out,[[right,low,s],[right,high,s],[left,high,s],[left,low,s]],
            [0.,0.,1.],[[uv.1,uv.3],[uv.1,v0],[uv.0,v0],[uv.0,uv.3]],
            crate::interior_models::authored_face_color([1.;4],Vec3::Z));
    }}
}

/// The lobby drawing is a low upholstered stool, not a backed link-club chair.
/// Four short feet and a closed cushion retain its own yellow/ochre palette;
/// only the cushion's upper drawing is mapped onto the horizontal top.
// Native stools are compact rectangular seats. The upper source row owns
// their cushion/frame; the lower row owns the two drawn feet and shadow.
fn native_red_bed_source(p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    p.kind==ModelKind::BedRed && p.width==2 && p.height==4
        && p.indices(g.width).enumerate().all(|(j,i)|crate::players_house::bed_local(&cells[i].source)
            ==Some(((j%2)as u8,(j/2)as u8)))
}

fn native_home_fixture_source(p: &Placement, cells: &[&VisualTile], g: &GridGeometry)
    -> Option<[[usize; 2]; 4]> {
    match (p.kind, p.width, p.height) {
        (ModelKind::Radio, 2, 2) if p.indices(g.width).enumerate().all(|(j,i)| {
            let source=&cells[i].source;
            source.tileset_id.as_ref()=="players_room" && source.metatile_id==1
                && source.subtile_column as usize==2+j%2 && source.subtile_row as usize==2+j/2
                && source.tile_index==[0x42,0x43,0x52,0x53][j]
        }) => Some([[1,1],[0,0],[5,4],[0,0]]),
        (ModelKind::KitchenStove | ModelKind::KitchenSink, 2, 2)
            if p.indices(g.width).enumerate().all(|(j,i)| {
                let source=&cells[i].source;
                source.metatile_id==7 && crate::players_house::upright_fixture_local(source)
                    ==Some(((j%2)as u8,(j/2)as u8,2,2))
                    && (source.subtile_column<2)==(p.kind==ModelKind::KitchenStove)
            }) => Some([[2,10],[2,3],[1,6],[0,0]]),
        (ModelKind::Refrigerator, 2, 3) if p.indices(g.width).enumerate().all(|(j,i)| {
            let source=&cells[i].source;
            source.metatile_id==15 && crate::players_house::upright_fixture_local(source)
                ==Some(((j%2)as u8,(j/2)as u8,2,3))
        }) => Some([[3,3],[2,1],[3,7],[3,3]]),
        (ModelKind::BedFeathery, 2, 4) if p.indices(g.width).enumerate().all(|(j,i)| {
            cells[i].source.metatile_id==0x1b && crate::interior::player_bed_local(&cells[i].source)
                ==Some(((j%2)as u8,(j/2)as u8))
        }) => Some([[5,2],[0,0],[5,2],[1,1]]),
        (ModelKind::Cabinet | ModelKind::DrawerCabinet, 2, 3) if p.indices(g.width).enumerate().all(|(j, i)| {
            let source = &cells[i].source;
            source.metatile_id == 0x11 && source.subtile_column >= 2
                && crate::players_house::upright_fixture_local(source)
                    == Some(((j % 2) as u8, (j / 2) as u8, 2, 3))
        }) => {
            // Packed drawer tile3a: dark recess, medium front, pale cap and
            // medium handle. Do not sample tile11's floor-only first row.
            Some([[1, 14], [2, 18], [2, 17], [1, 17]])
        }
        (ModelKind::GameConsole, 2, 2) if p.indices(g.width).enumerate().all(|(j, i)| {
            crate::players_house::console_local(&cells[i].source)
                == Some(((j % 2) as u8, (j / 2) as u8))
        }) => {
            // Packed tile4a contains black controls, dark shell and white
            // highlights. The original buttons have no red pigment.
            Some([[5, 0], [4, 4], [7, 1], [4, 4]])
        }
        _ => None,
    }
}

fn native_bookcase_room(map:&str)->bool {
    reviewed_native_room(map)||traditional_room::floor_map(map)
        ||matches!(map,"DragonShrine"|"EarlsPokemonAcademy"|"ElmsLab"|"FuchsiaGym"|"OaksLab")
}

fn native_bookcase_closed_base(p:&Placement,cells:&[&VisualTile],g:&GridGeometry,unit:usize)->bool {
    let i=(p.row+p.height-1)*g.width+p.column+unit*2;
    match cells[i].source.tileset_id.as_ref() {
        "house"|"players_house"|"traditional_house"=>true,
        "lab"=>cells[i].source.tile_index==0x13&&cells[i+1].source.tile_index==0x14,
        _=>false,
    }
}

/// The fitted cabinet supplies the exact footprint and upper shelves. Only a
/// complete source drawing can replace its lowest shelf with the source's own
/// cupboard/drawer front; Oak's three-row shelves retain the original mesh.
fn append_native_bookcase_base(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,
    cells:&[&VisualTile],unit:usize,first:usize,indices:usize,wood:[f32;2]) {
    if !native_bookcase_closed_base(p,cells,g,unit) {return;}
    let mut low=[f32::INFINITY;3];let mut high=[f32::NEG_INFINITY;3];
    for v in &out.positions[first..] {for axis in 0..3 {
        low[axis]=low[axis].min(v[axis]);high[axis]=high[axis].max(v[axis]);
    }}
    // The lower shelf ends at the first horizontal divider in both supported
    // canonical models. Retain upper vertices and triangles at their fitted
    // positions, including the full-height side frames.
    let top=low[1]+(high[1]-low[1])*0.4;
    let triangles=out.indices.split_off(indices);
    out.indices.extend(triangles.chunks_exact(3).filter(|tri|
        !tri.iter().all(|&i|out.positions[i as usize][1]<top-0.001))
        .flatten().copied());
    let body=out.positions.len();let body_indices=out.indices.len();
    // Sit inside the surviving side frames, avoiding coincident side faces.
    let inset=(high[0]-low[0])*0.025;
    architecture_box(out,[low[0]+inset,high[0]-inset,low[1],top,low[2]+inset,high[2]],[1.;3]);
    out.uvs[body..].fill(wood);
    // Replace the body's front with the two actual source cells. This retains
    // cupboard seams/handles and the distinct horizontal drawer fronts.
    out.indices.drain(body_indices+6..body_indices+12);
    for x in 0..2 {
        let uv=g.uv(p.column+unit*2+x,p.row+p.height-1);
        let w=low[0]+inset+(high[0]-low[0]-2.*inset)*x as f32/2.;
        let e=low[0]+inset+(high[0]-low[0]-2.*inset)*(x+1)as f32/2.;
        append_quad(out,[[e,low[1],high[2]],[e,top,high[2]],[w,top,high[2]],[w,low[1],high[2]]],
            [0.,0.,1.],[[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2],[uv.0,uv.3]],
            crate::interior_models::authored_face_color([1.;4],Vec3::Z));
    }
}

fn native_bookcase_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<[[usize;2];2]> {
    if !matches!(p.kind,ModelKind::Bookcase|ModelKind::LabBookcase)||p.width==0||p.height==0||p.width%2!=0
        ||g.width.checked_mul(g.height)!=Some(cells.len())
        ||p.width>g.width||p.height>g.height||p.column>g.width-p.width||p.row>g.height-p.height {return None;}
    let atlas=cells[p.row*g.width+p.column].source.tileset_id.as_ref();
    if atlas=="radio_tower" {
        return native_radio_room::bookcase_source(map,p,cells,g).then_some([[3,11],[3,12]]);
    }
    if !p.indices(g.width).all(|i|cells[i].source.tileset_id.as_ref()==atlas) {return None;}
    if atlas=="traditional_house" {
        if !traditional_room::floor_map(map) {return None;}
    } else if !native_bookcase_room(map) {return None;}
    let complete=match atlas {
        "house"=>p.height==4&&p.indices(g.width).enumerate().all(|(j,i)|
            crate::house::bookcase_local(&cells[i].source)==Some(((j%p.width%2)as u8,(j/p.width)as u8))),
        "players_house"=>p.height==4&&p.indices(g.width).enumerate().all(|(j,i)|
            crate::players_house::bookcase_local(&cells[i].source)==Some(((j%p.width%2)as u8,(j/p.width)as u8))),
        "lab"=>built_in_profiles().objects.iter().any(|o|o.tileset=="lab"&&profile_kind(o)==Some(ModelKind::LabBookcase)
            &&p.height==o.tiles.len()&&p.width==o.tiles[0].len()
            &&p.indices(g.width).enumerate().all(|(j,i)| {
                let s=&cells[i].source;
                s.metatile_id==o.metatile&&s.subtile_column as usize==o.origin[0]as usize+j%p.width
                    &&s.subtile_row as usize==o.origin[1]as usize+j/p.width&&s.tile_index==o.tiles[j/p.width][j%p.width]
            })),
        "players_room"|"mansion"|"traditional_house"=> (0..p.width/2).all(|unit|
            INTERIOR_DRAWINGS.iter().any(|d|d.kind==ModelKind::Bookcase&&d.tileset==atlas
                &&d.size==[2,p.height]&&(0..p.height).all(|y|(0..2).all(|x| {
                    let s=&cells[(p.row+y)*g.width+p.column+unit*2+x].source;
                    s.metatile_id==d.block&&s.subtile_column as usize==d.origin[0]as usize+x
                        &&s.subtile_row as usize==d.origin[1]as usize+y&&s.tile_index==d.tiles[y*2+x]
                })))),
        _=>false,
    };
    complete.then_some([if matches!(atlas,"lab"|"mansion"){[3,11]}else{[3,10]},[3,12]])
}

fn native_house_radio_source(p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    p.kind==ModelKind::Radio && p.width==2 && p.height==3
        && p.indices(g.width).enumerate().all(|(j,i)| {
            let s=&cells[i].source;
            s.metatile_id==0x1d && crate::house::upright_fixture_local(s)
                ==Some(((j%2)as u8,(j/2)as u8,2,3))
        })
}

fn native_computer_source(p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<[[usize;2];5]> {
    if p.kind!=ModelKind::Computer {return None;}
    let source=&cells[p.row*g.width+p.column].source;
    if source.tileset_id.as_ref()=="players_room" && p.width==2 && p.height==3
        && p.indices(g.width).enumerate().all(|(j,i)| {
            if j<4 {crate::interior::player_room_pc_monitor_local(&cells[i].source)==Some(((j%2)as u8,(j/2)as u8))}
            else {crate::interior::player_room_pc_keyboard_local(&cells[i].source)==Some(((j%2)as u8,0))}
        }) {return Some([[3,0],[0,0],[3,1],[0,0],[0,0]]);}
    let drawing=INTERIOR_DRAWINGS.iter().find(|d|d.kind==ModelKind::Computer
        && matches!(d.tileset,"house"|"players_house") && source.tileset_id.as_ref()==d.tileset
        && d.size==[p.width,p.height] && source.metatile_id==d.block
        && p.indices(g.width).enumerate().all(|(j,i)| {
            let s=&cells[i].source;
            s.tileset_id.as_ref()==d.tileset && s.metatile_id==d.block
                && s.subtile_column==d.origin[0]+(j%p.width)as u8
                && s.subtile_row==d.origin[1]+(j/p.width)as u8 && s.tile_index==d.tiles[j]
        }))?;
    Some(if drawing.tileset=="house" {
        // Tile40 has a blue cap, tile20 gray electronics, tile42 gold timber.
        // Sample each part's own palette rather than painting the stand blue.
        [[3,0],[3,8],[3,1],[4,20],[0,0]]
    } else if drawing.tiles[0]==0x11 {
        [[7,0],[0,0],[14,0],[0,0],[0,0]]
    } else {[[3,0],[0,0],[3,1],[0,0],[0,0]]})
}

fn append_native_station_ticket_boxes(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,
    selected:&crate::interior_models::Model,b:[f32;4]) {
    let scale=g.tile_height/8.;let [_,e,n,_]=b;
    for unit in 0..4 {
        let sample=|x:usize,y:usize| {let uv=g.uv(p.column+x/8,p.row+unit*2+y/8);
            [uv.0+(uv.1-uv.0)*((x%8)as f32+0.5)/8.,uv.2+(uv.3-uv.2)*((y%8)as f32+0.5)/8.]};
        let pixels=if unit==3 {[[11,3],[2,2],[0,1],[3,1],[2,1]]}
            else {[[11,3],[1,2],[0,1],[2,2],[3,1]]};
        let swatches=pixels.map(|v|sample(v[0],v[1]));
        let first=out.positions.len();
        assert!(selected.append_native_ticket_box(out,[0.,14.*scale,0.,10.*scale],20.*scale,swatches));
        // Face west into the waiting room; each source module owns 16px of bank.
        for v in &mut out.positions[first..] {let [x,y,z]=*v;*v=[e-z,y,n+unit as f32*16.*scale+x+scale];}
        for v in &mut out.normals[first..] {let [x,y,z]=*v;*v=[-z,y,x];}
        for (normal,color) in out.normals[first..].iter().zip(&mut out.colors[first..]) {
            *color=crate::interior_models::authored_face_color([1.;4],Vec3::from_array(*normal));
        }
    }
}

fn append_native_station_wall(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4],pillar:bool) {
    let [w,e,n,s]=b;let height=p.height_pixels*g.tile_height/8.;
    let sample=|x:usize,y:usize| {
        let uv=g.uv(p.column+x/8,p.row+y/8);
        [uv.0+(uv.1-uv.0)*((x%8)as f32+0.5)/8.,uv.2+(uv.3-uv.2)*((y%8)as f32+0.5)/8.]
    };
    let gray=sample(0,32);let blue=sample(17,1);let joint=sample(19,3);let cream=sample(2,2);
    let first=out.positions.len();let index=out.indices.len();
    architecture_box(out,[w,e,0.,height,n,s],[1.;3]);out.uvs[first..].fill(gray);
    let faces=out.indices.split_off(index);
    out.indices.extend(faces.chunks_exact(3).filter(|tri|out.normals[tri[0]as usize][2]<0.5)
        .flatten().copied());
    let point=|x:f32,y:f32|[w+(e-w)*x,height*(1.-y),s];
    // A closed gray lower wall joins a blue upper masonry course. The front
    // samples the same live source pigments, without floor-sized brick cards.
    native_source_surface_patches(out,&[0.,0.025,0.5,0.975,1.],&[0.,0.48,0.50,0.525,0.75,1.],
        point,|x,y| {let y=1.-y;if y<0.50 {gray}else if pillar&&x<0.5 {cream}
            else if x<0.025||x>0.975||y<0.525 {joint}else{blue}},
        |_,_|1.,true);
}

fn append_native_station_railing(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4]) {
    let [w,e,n,s]=b;let scale=g.tile_height/8.;let uv=g.uv(p.column,p.row);
    let paint=[uv.0+(uv.1-uv.0)*1.5/8.,uv.2+(uv.3-uv.2)*1.5/8.];
    let first=out.positions.len();
    for x in [w,(w+e)*0.5,e-scale] {
        architecture_box(out,[x,x+scale,0.,11.*scale,n,s],[1.;3]);
    }
    for y in [3.,7.,10.] {
        architecture_box(out,[w,e,y*scale,(y+1.)*scale,n,s],[1.;3]);
    }
    out.uvs[first..].fill(paint);
}

// The player-home seat is a round wooden stool: a beveled solid seat and
// four timber feet, using the gold/brown texels from its own complete drawing.
fn append_native_round_wood_stool(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,
    bounds:[f32;4]) {
    let [w,e,n,s]=bounds;let scale=g.tile_height/8.;
    let center=Vec3::new((w+e)*0.5,0.,(n+s)*0.5);
    let paint=native_source_swatches(p,g,[[4,2],[6,9]]);
    for x in [-2.2,2.2] {for z in [-2.2,2.2] {
        let first=out.positions.len();
        architecture_box(out,[center.x+(x-0.5)*scale,center.x+(x+0.5)*scale,
            0.,3.3*scale,center.z+(z-0.5)*scale,center.z+(z+0.5)*scale],[1.;3]);
        out.uvs[first..].fill(paint[1]);
    }}
    let radius=((e-w)*0.5).min((s-n)*0.5-scale);
    let rings=[(0.,3.),(radius-0.3*scale,3.),(radius,3.3),
        (radius,4.7),(radius-0.3*scale,5.),(0.,5.)];
    for pair in rings.windows(2) {for i in 0..32 {
        let point=|(r,y):(f32,f32),step:usize| {
            let a=std::f32::consts::TAU*step as f32/32.;
            center+Vec3::new(r*a.cos(),y*scale,r*a.sin())
        };
        let a=point(pair[0],i);let b=point(pair[1],i);
        let c=point(pair[1],i+1);let d=point(pair[0],i+1);
        for points in [[a,b,c],[a,c,d]] {
            let normal=(points[1]-points[0]).cross(points[2]-points[0]);
            if normal.length_squared()<1e-12 {continue;}
            let normal=normal.normalize();let first=out.positions.len() as u32;
            out.positions.extend(points.map(|v|v.to_array()));
            out.normals.extend([normal.to_array();3]);out.uvs.extend([paint[0];3]);
            out.colors.extend([crate::interior_models::authored_face_color([1.;4],normal);3]);
            out.indices.extend([first,first+1,first+2]);
        }
    }}
}

fn append_native_rectangular_stool(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,
    bounds:[f32;4]) {
    let [left,right,north,south]=bounds;let scale=g.tile_height/8.;
    let [w,e,n,s]=[left,right,north+scale,south-scale];
    let texel=|x:usize,y:usize| {
        let uv=g.uv(p.column+x/8,p.row+y/8);
        [uv.0+(uv.1-uv.0)*((x%8) as f32+0.5)/8.,
         uv.2+(uv.3-uv.2)*((y%8) as f32+0.5)/8.]
    };
    let foot=texel(6,9);let side=texel(4,2);
    for x in [w+scale,e-2.*scale] {for z in [n+scale,s-2.*scale] {
        let first=out.positions.len();
        architecture_box(out,[x,x+scale,0.,3.*scale,z,z+scale],[1.;3]);
        out.uvs[first..].fill(foot);
    }}
    let first=out.positions.len();let indices=out.indices.len();
    architecture_box(out,[w,e,3.*scale,5.*scale,n,s],[1.;3]);
    out.uvs[first..].fill(side);out.indices.drain(indices..indices+6);
    // Crop only the seat drawing, keeping the atlas seam between02 and03.
    // The lower12/13 foot drawing does not get stretched across upholstery.
    for x in 0..2 {
        let sx0=(x*8).max(1);let sx1=((x+1)*8).min(15);
        let a=w+(e-w)*(sx0-1) as f32/14.;let b=w+(e-w)*(sx1-1) as f32/14.;
        let uv=g.uv(p.column+x,p.row);
        let u0=uv.0+(uv.1-uv.0)*(sx0-x*8) as f32/8.;
        let u1=uv.0+(uv.1-uv.0)*(sx1-x*8) as f32/8.;
        let v0=uv.2+(uv.3-uv.2)/8.;let v1=uv.2+(uv.3-uv.2)*7./8.;
        append_quad(out,[[a,5.*scale,n],[a,5.*scale,s],[b,5.*scale,s],[b,5.*scale,n]],
            [0.,1.,0.],[[u0,v0],[u0,v1],[u1,v1],[u1,v0]],[1.;4]);
    }
}

fn append_native_center_lounge_seat(out: &mut SurfaceMeshData,g: &GridGeometry,
    p: &Placement,bounds: [f32;4],foot_pixel: [usize;2]) {
    let [w,e,n,s]=bounds;let scale=g.tile_height/8.;
    let texel=|x:usize,y:usize| {
        let uv=g.uv(p.column+x/8,p.row+y/8);
        [uv.0+(uv.1-uv.0)*((x%8) as f32+0.5)/8.,
         uv.2+(uv.3-uv.2)*((y%8) as f32+0.5)/8.]
    };
    let foot=texel(foot_pixel[0],foot_pixel[1]);let side=texel(1,3);
    for x in [w+scale,e-3.*scale] {for z in [n+scale,s-3.*scale] {
        let first=out.positions.len();
        architecture_box(out,[x,x+2.*scale,0.,3.*scale,z,z+2.*scale],[1.;3]);
        out.uvs[first..].fill(foot);
    }}
    let first=out.positions.len();let indices=out.indices.len();
    architecture_box(out,[w,e,3.*scale,7.*scale,n,s],[1.;3]);
    out.uvs[first..].fill(side);
    // The original lower four pixels describe feet, not cushion upholstery.
    // Replace the box top with the cropped native upper pattern, conserving
    // source tile UVs independently on either side of each atlas boundary.
    out.indices.drain(indices..indices+6);
    for y in 0..2 {for x in 0..2 {
        let sx0=(x*8).max(1);let sx1=((x+1)*8).min(15);
        let sy0=(y*8).max(1);let sy1=((y+1)*8).min(11);
        let left=w+(e-w)*(sx0-1) as f32/14.;
        let right=w+(e-w)*(sx1-1) as f32/14.;
        let north=n+(s-n)*(sy0-1) as f32/10.;
        let south=n+(s-n)*(sy1-1) as f32/10.;
        let uv=g.uv(p.column+x,p.row+y);
        let u0=uv.0+(uv.1-uv.0)*(sx0-x*8) as f32/8.;
        let u1=uv.0+(uv.1-uv.0)*(sx1-x*8) as f32/8.;
        let v0=uv.2+(uv.3-uv.2)*(sy0-y*8) as f32/8.;
        let v1=uv.2+(uv.3-uv.2)*(sy1-y*8) as f32/8.;
        append_quad(out,[[left,7.*scale,north],[left,7.*scale,south],
            [right,7.*scale,south],[right,7.*scale,north]],[0.,1.,0.],
            [[u0,v0],[u0,v1],[u1,v1],[u1,v0]],[1.;4]);
    }}
}

fn native_wise_divider(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
    p.kind==ModelKind::WallTraditional&&p.indices(g.width).enumerate().all(|(j,i)|
        crate::wise_trios::divider_local(map,&cells[i].source)
            ==Some(((j%p.width)as u8,(j/p.width)as u8,p.width,p.height)))
}

fn append_wise_divider(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,cells:&[&VisualTile],[w,e,n,s]:[f32;4]) {
    // The two source rows draw a short post in perspective. Extend its timber
    // construction, not the entire post picture repeated up the wall.
    let source=&cells[p.row*g.width+p.column].source;
    let paint=native_source_swatches(p,g,if source.metatile_id==3 {
        [[1,0],[2,0],[3,1],[3,0]]
    }else{[[0,0],[2,0],[3,1],[3,0]]});
    let k=g.tile_height/8.;let height=32.*k;
    if p.width==2 {
        // These paired post pictures describe a north/south return. Join its
        // authored segments across the intervening perspective floor rows.
        let mut start=p.row;
        if source.metatile_id==0x38&&p.row>=4 {
            let previous=&cells[(p.row-4)*g.width+p.column].source;
            let gap=(p.row-2..p.row).all(|y|(p.column..p.column+2).all(|x| {
                let s=&cells[y*g.width+x].source;
                s.tileset_id.as_ref()=="traditional_house"&&s.metatile_id==0x38&&s.tile_index==0x50
            }));
            if gap&&crate::wise_trios::divider_local("WiseTriosRoom",previous)==Some((0,0,2,2)) {
                start-=2;
            }
        }
        let mut end=p.row+p.height;
        if source.metatile_id==0x38&&end+2<=g.height&&p.column>=2 {
            let closes=(0..2).all(|y|(0..4).all(|x| {
                crate::wise_trios::divider_local("WiseTriosRoom",&cells[(end+y)*g.width+p.column-2+x].source)
                    ==Some((x as u8,y as u8,4,2))
            }));
            if closes {end+=2;}
        }
        let back=g.origin_z+end as f32*g.tile_height;
        let first=out.positions.len();let front=g.origin_z+start as f32*g.tile_height;
        architecture_box(out,[e-3.*k,e,0.,height,front,back],[1.;3]);
        out.uvs[first..].fill(paint[0]);
        for (low,high) in [(0.,1.2),(30.8,32.)] {
            let first=out.positions.len();architecture_box(out,[e-3.1*k,e+0.05*k,low*k,high*k,front,back],[1.;3]);
            out.uvs[first..].fill(paint[1]);
        }
        return;
    }
    let timber=|out:&mut SurfaceMeshData,bounds,uv| {
        let first=out.positions.len();architecture_box(out,bounds,[1.;3]);
        out.uvs[first..].fill(uv);
    };
    let indices=out.indices.len();
    timber(out,[w,e,1.2*k,30.8*k,n,s-0.35*k],paint[0]);
    out.indices.drain(indices+6..indices+12);
    let xs=(0..=32).map(|i|i as f32/32.).collect::<Vec<_>>();
    let ys=(0..=12).map(|i|i as f32/12.).collect::<Vec<_>>();
    native_source_surface_patches(out,&xs,&ys,
        |x,y|[w+(e-w)*x,(30.8-29.6*y)*k,s-0.35*k],|_,_|paint[0],
        |x,y|1.+0.055*crate::scenery_finish::timber_grain([x*(e-w)/k,y*32.,0.]),true);
    for column in (0..=p.width).step_by(2) {
        let x=w+column as f32*g.tile_width;
        timber(out,[(x-0.6*k).max(w),(x+0.6*k).min(e),0.,height,n,s],paint[1]);
    }
    for (low,high) in [(0.,1.2),(30.8,32.)] {
        timber(out,[w,e,low*k,high*k,n,s+0.05*k],paint[1]);
    }
    // Fine vertical board seams remain continuous between the frame rails.
    for column in 1..p.width {
        let x=w+column as f32*g.tile_width;
        timber(out,[x-0.08*k,x+0.08*k,1.2*k,30.8*k,s-0.35*k,s-0.32*k],paint[1]);
    }
}

/// Closed native domestic wall course, selected only from complete house art.
fn append_kanto_house_wall(out: &mut SurfaceMeshData,g: &GridGeometry,p: &Placement,
    bounds: [f32;4]) {
    let [west,east,north,south]=bounds;
    // Source rows describe the finish, not the room's physical ceiling height.
    // Repeat native material courses rather than stretching their texels.
    let base=p.base_pixels*g.tile_height/8.;
    let height=(p.base_pixels+p.height_pixels).max(32.)*g.tile_height/8.;
    let first=out.positions.len();let index_first=out.indices.len();
    architecture_box(out,[west,east,base,height,north,south],[1.;3]);
    let uv=g.uv(p.column,p.row);
    let body=[uv.0+(uv.1-uv.0)*4.5/8.,uv.2+(uv.3-uv.2)*2.5/8.];
    out.uvs[first..].fill(body);
    // Replace the plain front with the source tiles, without coplanar faces.
    out.indices.drain(index_first+6..index_first+12);
    let rows=((height-base)/g.tile_height).ceil()as usize;
    for y in 0..rows {for x in 0..p.width {
        let left=west+x as f32*g.tile_width;let right=left+g.tile_width;
        let top=height-y as f32*g.tile_height;let bottom=(top-g.tile_height).max(base);
        let uv=g.uv(p.column+x,p.row+y%p.height);
        append_quad(out,[[right,bottom,south],[right,top,south],
            [left,top,south],[left,bottom,south]],[0.,0.,1.],
            [[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2],[uv.0,uv.3]],
            crate::interior_models::authored_face_color([1.;4],Vec3::Z));
    }}
}

fn append_native_house_window(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,cells:&[&VisualTile],b:[f32;4]) {
    let [w,e,n,s]=b;let index=out.indices.len();
    // Reuse the closed domestic wall volume and replace only its front course.
    append_kanto_house_wall(out,g,p,[w,e,n,s]);
    out.indices.truncate(index+5*6);
    let (u0,_,v0,_)=g.uv(p.column,p.row);
    let (_,u1,_,v1)=g.uv(p.column+p.width-1,p.row+p.height-1);
    let course=p.height as f32*g.tile_height;
    let height=(p.base_pixels+p.height_pixels).max(32.)*g.tile_height/8.;
    // Keep one window at its source size near the ceiling, with wall below.
    let atlas=&cells[p.row*g.width+p.column].source.tileset_id;
    let wall_tile=if atlas.as_ref()=="house"{0}else if atlas.as_ref()=="players_house"{0x11}else{2};
    let donor=cells.iter().position(|cell|cell.source.tileset_id==*atlas&&cell.source.tile_index==wall_tile);
    let uv=donor.map(|i|g.uv(i%g.width,i/g.width)).unwrap_or_else(||g.uv(p.column,p.row+p.height-1));
    let body=[uv.0+(uv.1-uv.0)*4.5/8.,uv.2+(uv.3-uv.2)*2.5/8.];
    append_quad(out,[[e,0.,s],[e,height-course,s],[w,height-course,s],[w,0.,s]],
        [0.,0.,1.],[body;4],crate::interior_models::authored_face_color([1.;4],Vec3::Z));
    let relief=g.tile_height*0.06;
    assert!(height>0. && e>w && s>n,"native curtain wall requires complete source bounds");
    let point=|x:f32,y:f32| {
        let inset=if x==0.||x==1.||y==0.||y==1. {0.}
            else {relief*(0.75+0.25*(x*std::f32::consts::TAU*4.).cos())};
        [w+(e-w)*x,height-course*y,s-inset]
    };
    // The original curtains and neighboring frame keep their exact UVs,
    // while recessed folds add depth without detaching from the wall course.
    native_source_surface_patches(out,&[0.,0.125,0.25,0.375,0.5,0.625,0.75,0.875,1.],
        &[0.,0.5,1.],point,|x,y|[u0+(u1-u0)*x,v0+(v1-v0)*y],|_,_|1.,false);
}

#[cfg(test)]
mod kanto_native_wall_tests {
    use super::*;
    #[test]
    fn chairs_face_aligned_nearby_tables_without_turning_backless_stools() {
        let seat=Placement{kind:ModelKind::Chair,column:8,row:8,width:2,height:2,
            ground:0,depth_pixels:8.,height_pixels:11.,base_pixels:0.,footing_pixels:None};
        for (column,row,facing) in [(8,12,0),(12,8,1),(8,2,2),(2,8,3)] {
            let table=Placement{kind:ModelKind::DiningTable,column,row,width:4,height:4,..seat.clone()};
            assert_eq!(chair_table_facing(&seat,&[table.clone()],[0,0]),Some(facing));
            let stool=Placement{kind:ModelKind::Stool,..seat.clone()};
            assert_eq!(chair_table_facing(&stool,&[table],[0,0]),None);
        }
        let distant=Placement{kind:ModelKind::DiningTable,row:20,..seat.clone()};
        let diagonal=Placement{kind:ModelKind::DiningTable,row:12,column:12,..seat.clone()};
        let cabinet=Placement{kind:ModelKind::Cabinet,row:12,..seat.clone()};
        assert_eq!(chair_table_facing(&seat,&[distant,diagonal,cabinet],[0,0]),None);
        let student=Placement{row:10,..seat.clone()};
        let own=Placement{kind:ModelKind::DiningTable,row:8,width:4,height:2,..seat.clone()};
        let next=Placement{row:12,..own.clone()};
        assert_eq!(chair_table_facing(&student,&[next.clone(),own.clone()],[0,0]),Some(2));
        // Camera grids can begin at any source phase. Translating the same
        // packed rows must never pair a chair with the next student's desk.
        for x in 0..4 {for y in 0..4 {
            let shift=|mut p:Placement|{p.column+=x;p.row+=y;p};
            let moved=shift(student.clone());
            let phase=[(4-x)%4,(4-y)%4];
            let source=VisualTileSource{tileset_id:"lab".into(),metatile_id:0x16,
                subtile_column:0,subtile_row:2,tile_index:0x40};
            assert_eq!(chair_source_phase(&moved,&source),phase);
            assert_eq!(chair_table_facing(&moved,&[shift(next.clone()),shift(own.clone())],phase),Some(2));
        }}
    }
    #[test]
    fn external_wise_room_has_connected_ceiling_height_timber_returns() {
        let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(std::path::PathBuf::from)
            .unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        if !path.is_file(){return;}
        let pack=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let module=&pack.pack().data().maps["WiseTriosRoom"];
        let art=&pack.pack().runtime_files()["data/tilesets/traditional_house_metatiles.bin"];
        let (w,h)=(module.attributes.width as usize*4,module.attributes.height as usize*4);
        let tiles=(0..w*h).map(|i| {
            let (x,y)=(i%w,i/w);let b=module.blocks[y/4*(w/4)+x/4];
            VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                source:VisualTileSource{tileset_id:"traditional_house".into(),metatile_id:b,
                    subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,
                    tile_index:art[b as usize*16+y%4*4+x%4]as u16}}
        }).collect::<Vec<_>>();
        let cells=tiles.iter().collect::<Vec<_>>();
        let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let placements=resolve("WiseTriosRoom",&cells,&g,None);
        let walls=placements.iter().filter(|p|native_wise_divider("WiseTriosRoom",p,&cells,&g)).collect::<Vec<_>>();
        assert_eq!(walls.len(),10,"four north courses, four crosspieces and two returns");
        for p in walls {
            let mut mesh=TerrainMeshData{footing_heights:vec![7.;w*h],..Default::default()};
            let mut claimed=vec![false;w*h];
            assert!(append(&mut mesh,"WiseTriosRoom",&cells,&g,p,&mut claimed,[0,0]));
            assert_eq!(mesh.footing_heights,vec![7.;w*h]);
            let object=&mesh.textured.positions[p.width*p.height*4..];
            assert_eq!(object.iter().map(|v|v[1]).fold(0.,f32::max),32.);
            if p.width==2 {
                let min_z=object.iter().map(|v|v[2]).fold(f32::INFINITY,f32::min);
                let max_z=object.iter().map(|v|v[2]).fold(f32::NEG_INFINITY,f32::max);
                assert_eq!((min_z,max_z),if p.row==6 {(48.,64.)}else{(64.,112.)});
                assert!(object.iter().all(|v|v[0]>=60.8&&v[0]<=64.1));
            }
        }
    }
    #[test]
    fn wise_divider_reaches_ceiling_with_native_source_and_unchanged_opening() {
        let g=GridGeometry{width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let tiles=(0..16).map(|i|VisualTile{column:(i%4)as u32,row:(i/4)as u32,
            texture:Default::default(),animation_frames:None,priority:false,
            source:VisualTileSource{tileset_id:"traditional_house".into(),metatile_id:0x28,
                subtile_column:(i%4)as u8,subtile_row:(i/4)as u8,
                tile_index:if i<4 {0x40}else if i<8 {0x41}else{0x50}}}).collect::<Vec<_>>();
        let cells=tiles.iter().collect::<Vec<_>>();let placements=resolve("WiseTriosRoom",&cells,&g,None);
        let p=placements.iter().find(|p|native_wise_divider("WiseTriosRoom",p,&cells,&g)).unwrap();
        assert_eq!((p.width,p.height,p.height_pixels),(4,2,32.));
        let mut mesh=TerrainMeshData{footing_heights:vec![72.;16],..Default::default()};
        let mut claimed=vec![false;16];assert!(append(&mut mesh,"WiseTriosRoom",&cells,&g,p,&mut claimed,[0,0]));
        assert!(mesh.textured.positions.iter().any(|v|v[1]==32.));
        assert_eq!(mesh.footing_heights,vec![72.;16]);
        assert_eq!(claimed,(0..16).map(|i|i<8).collect::<Vec<_>>());
        let range=p.width*p.height*4..mesh.textured.positions.len();
        assert_eq!(mesh.textured.cutaway_ranges,vec![range.clone()]);
        assert_eq!(mesh.layered_alpha_textured_ranges,vec![range]);
        assert!(mesh.depth_coverage_textured_ranges.is_empty(),"timber reveal must not become a wire fence");
        assert!(resolve("DanceTheater",&cells,&g,None).iter().all(|p|!native_wise_divider("DanceTheater",p,&cells,&g)));
        for i in 0..8 {
            let mut altered=tiles.clone();altered[i].source.tile_index=0xff;
            let refs=altered.iter().collect::<Vec<_>>();
            assert!(!resolve("WiseTriosRoom",&refs,&g,None).iter().any(|p|native_wise_divider("WiseTriosRoom",p,&refs,&g)));
        }
    }
    #[test]
    fn kanto_native_house_wall_keeps_closed_source_course_and_shared_seams() {
        let g=GridGeometry{width:8,height:2,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let p=Placement{kind:ModelKind::WallDomestic,column:0,row:0,width:4,height:2,
            ground:0,depth_pixels:3.,height_pixels:24.,base_pixels:0.,footing_pixels:None};
        let mut wall=SurfaceMeshData::default();
        append_kanto_house_wall(&mut wall,&g,&p,[0.,32.,13.,16.]);
        assert_eq!(wall.indices.len(),(5+16)*6,"five closed sides and sixteen source front tiles");
        for ids in wall.indices.chunks_exact(3) {
            let [a,b,c]=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(wall.positions[i as usize]));
            let normal=Vec3::from_array(wall.normals[ids[0]as usize]);
            assert!((b-a).cross(c-a).dot(normal)>0.,"outward closed faces");
        }
        assert!(wall.positions.iter().all(|v|v[1]>=0. && v[1]<=32.));
        assert!(wall.positions.iter().any(|v|v[1]==32.));
        assert_eq!(proportions(ModelKind::WindowWall,2,2).1,32.);
        let mut adjacent=p.clone();adjacent.column=4;
        let mut other=SurfaceMeshData::default();
        append_kanto_house_wall(&mut other,&g,&adjacent,[32.,64.,13.,16.]);
        let seam=|mesh:&SurfaceMeshData| {
            let mut vertices=mesh.positions.iter().filter(|v|v[0]==32.).map(|v|(v[1]as i32,v[2]as i32)).collect::<Vec<_>>();
            vertices.sort();vertices.dedup();vertices
        };
        assert_eq!(seam(&wall),seam(&other));
        assert!(wall.uvs.iter().any(|uv|uv[0]>0. && uv[1]>0.));
    }
}

/// The native row is a back-to-back bank: seats lie west and east, so
/// each cabinet's screen and controls face the seat on its own side.
fn append_outward_arcade_pair(mesh: &mut SurfaceMeshData, bounds: [f32; 4], base: f32, rise: f32) {
    let [west, east, north, south] = bounds;
    let half = (east - west) * 0.5;
    for (unit, facing) in [(0, -1.0), (1, 1.0)] {
        append_side_arcade(mesh, [west + half * unit as f32, west + half * (unit + 1) as f32, north, south], base, rise, facing);
    }
}

// A closed display case with native glass/equipment fronts and recessed bays.
// Complete source drawings choose its original merchandise, not a generic book model.
fn append_store_vending_roof(mesh:&mut SurfaceMeshData,first:usize,height:f32,scale:f32,swatches:[[f32;2];5])->bool {
    let roof=|tri:&[u32]|tri.iter().all(|&i|i as usize>=first
        &&mesh.normals[i as usize][1]>0.999
        &&(mesh.positions[i as usize][1]-height).abs()<0.001*scale);
    let points=mesh.indices.chunks_exact(3).filter(|tri|roof(tri))
        .flat_map(|tri|tri.iter().map(|&i|mesh.positions[i as usize])).collect::<Vec<_>>();
    if points.is_empty() {return false;}
    let [w,e,n,s]=[
        points.iter().map(|p|p[0]).fold(f32::INFINITY,f32::min),
        points.iter().map(|p|p[0]).fold(f32::NEG_INFINITY,f32::max),
        points.iter().map(|p|p[2]).fold(f32::INFINITY,f32::min),
        points.iter().map(|p|p[2]).fold(f32::NEG_INFINITY,f32::max),
    ];
    if e<=w||s<=n {return false;}
    let indices=std::mem::take(&mut mesh.indices);
    mesh.indices=indices.chunks_exact(3).filter(|tri|!roof(tri)).flatten().copied().collect();
    let xs=[0.,0.095,0.105,0.20,0.80,0.895,0.905,1.];
    let slots=std::array::from_fn::<_,6,_>(|i|[0.18+i as f32*0.10,0.225+i as f32*0.10]);
    let mut zs=vec![0.,0.085,0.095,0.905,0.915,1.];
    for [a,b] in slots {zs.extend([a,b]);}
    zs.sort_by(f32::total_cmp);
    let top=|mesh:&mut SurfaceMeshData,bounds:[f32;4],y:f32,uv:[f32;2]| {
        append_top(mesh,bounds,y,(uv[0],uv[0],uv[1],uv[1]));
    };
    for z in zs.windows(2) {for x in xs.windows(2) {
        let cx=(x[0]+x[1])*0.5;let cz=(z[0]+z[1])*0.5;
        if cx>0.20&&cx<0.80&&slots.iter().any(|[a,b]|cz>*a&&cz<*b) {continue;}
        let seam=((0.095..0.105).contains(&cx)||(0.895..0.905).contains(&cx))&&(0.085..0.915).contains(&cz)
            ||((0.085..0.095).contains(&cz)||(0.905..0.915).contains(&cz))&&(0.095..0.905).contains(&cx);
        let panel=(0.105..0.895).contains(&cx)&&(0.095..0.905).contains(&cz);
        top(mesh,[w+(e-w)*x[0],w+(e-w)*x[1],n+(s-n)*z[0],n+(s-n)*z[1]],height,
            swatches[if seam {1}else if panel {3}else{0}]);
    }}
    let [a,b]=[w+(e-w)*0.20,w+(e-w)*0.80];
    let floor=height-0.22*scale;
    for [c,d] in slots.map(|[c,d]|[n+(s-n)*c,n+(s-n)*d]) {
        top(mesh,[a,b,c,d],floor,swatches[2]);
        for (points,normal) in [
            ([[a,height,c],[a,floor,c],[b,floor,c],[b,height,c]],[0.,0.,1.]),
            ([[a,floor,d],[a,height,d],[b,height,d],[b,floor,d]],[0.,0.,-1.]),
            ([[a,floor,c],[a,height,c],[a,height,d],[a,floor,d]],[1.,0.,0.]),
            ([[b,height,c],[b,floor,c],[b,floor,d],[b,height,d]],[-1.,0.,0.]),
        ] {
            let start=mesh.positions.len();
            append_solid_quad(mesh,points,normal,
                crate::interior_models::authored_face_color([1.;4],Vec3::from_array(normal)));
            mesh.uvs[start..].fill(swatches[1]);
        }
    }
    true
}

fn append_native_retail_display(mesh:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4],blue_case:bool) {
    if blue_case {
        append_native_blue_case(mesh,g,p,b);
        return;
    }
    // The first eight source pixels describe the visible roof, including its
    // vents. Only the remaining three rows form the vertical door/display.
    let [w,e,n,s]=b;let height=p.height_pixels*0.75*g.tile_height/8.;
    let sample=|x:usize,y:usize| {
        let uv=g.uv(p.column+x/8,p.row+y/8);
        [uv.0+(uv.1-uv.0)*((x%8)as f32+0.5)/8.,
         uv.2+(uv.3-uv.2)*((y%8)as f32+0.5)/8.]
    };
    let first=mesh.positions.len();let index=mesh.indices.len();
    architecture_box(mesh,[w,e,0.,height,n,s],[1.;3]);
    // Paint the closed casing from this display, with its native front split
    // at the original cell boundaries. No book merchandise is substituted.
    let casing=if blue_case {
        // Blue aisle units have a mid-blue color at the retained source anchor;
        // electronics keep their original light cap. The native texture still
        // paints each individual glass/front section over the closed geometry.
        sample(6,3)
    } else {sample(1,1)};
    for i in first..mesh.positions.len() {mesh.uvs[i]=casing;}
    let faces=mesh.indices.split_off(index);
    mesh.indices.extend(faces.chunks_exact(3)
        .filter(|tri|mesh.normals[tri[0]as usize][2]<0.5
            &&mesh.normals[tri[0]as usize][1]<0.5).flatten().copied());
    for x in 0..p.width {
        append_top(mesh,[w+(e-w)*x as f32/p.width as f32,
            w+(e-w)*(x+1)as f32/p.width as f32,n,s],height,g.uv(p.column+x,p.row));
    }
    // Recessed bays meet raised shelf lips and a beveled outer frame.
    // All shared knots coincide: the inset front seals the same casing rather
    // than putting floating graphics or intersecting boxes in front of it.
    let xs=[0.,0.06,0.12,0.5,0.88,0.94,1.];
    let ys:&[f32]=if p.kind==ModelKind::RetailRefrigerator {
        &[0.25,0.29,0.5,0.75,0.95,1.]
    } else {&[0.25,0.29,0.45,0.5,0.54,0.70,0.75,0.79,0.95,1.]};
    let (u0,_,v0,_)=g.uv(p.column,p.row);
    let (_,u1,_,v1)=g.uv(p.column+p.width-1,p.row+p.height-1);
    let point=|x:f32,y:f32| {
        let recessed=if p.kind==ModelKind::RetailRefrigerator {y>0.25 && y<1.}
            else {![0.,0.25,0.5,0.75,1.].contains(&y)};
        let inset=if x>=0.12 && x<=0.88 && recessed {0.8*g.tile_height/8.} else {0.};
        [w+(e-w)*x,height*(1.-y)/0.75,s-inset]
    };
    for y in 0..ys.len()-1 {for x in 0..xs.len()-1 {
        let corners=[[xs[x],ys[y]],[xs[x],ys[y+1]],
            [xs[x+1],ys[y+1]],[xs[x+1],ys[y]]];
        for tri in [[0,1,2],[0,2,3]] {
            let coords=tri.map(|i|corners[i]);let points=coords.map(|[x,y]|point(x,y));
            let normal=(Vec3::from_array(points[1])-Vec3::from_array(points[0]))
                .cross(Vec3::from_array(points[2])-Vec3::from_array(points[0])).normalize();
            let first=mesh.positions.len()as u32;
            mesh.positions.extend(points);mesh.normals.extend([normal.to_array();3]);
            mesh.uvs.extend(coords.map(|[x,y]|[u0+(u1-u0)*x,v0+(v1-v0)*y]));
            mesh.colors.extend([crate::interior_models::authored_face_color([1.;4],normal);3]);
            mesh.indices.extend([first,first+1,first+2]);
        }
    }}
}

fn append_native_blue_case(mesh:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4]) {
    let [w,e,_,s]=b;let scale=g.tile_height/8.;let n=s-24.*scale;
    let height=8.*scale;
    let sample=|x:usize,y:usize| {
        let uv=g.uv(p.column+x/8,p.row+y/8);
        [uv.0+(uv.1-uv.0)*((x%8)as f32+0.5)/8.,
         uv.2+(uv.3-uv.2)*((y%8)as f32+0.5)/8.]
    };
    let first=mesh.positions.len();let index=mesh.indices.len();
    architecture_box(mesh,[w,e,0.,height,n,s],[1.;3]);
    mesh.uvs[first..].fill(sample(6,3));
    let faces=mesh.indices.split_off(index);
    mesh.indices.extend(faces.chunks_exact(3).filter(|tri|
        mesh.normals[tri[0]as usize][1]<0.5).flatten().copied());
    // Three source rows describe the long lid. The remaining perspective
    // strip must not turn into a tall ribbed cabinet door or corrugated side.
    for y in 0..3 {for x in 0..2 {
        append_top(mesh,[w+(e-w)*x as f32/2.,w+(e-w)*(x+1)as f32/2.,
            n+8.*scale*y as f32,n+8.*scale*(y+1)as f32],height,
            g.uv(p.column+x,p.row+y));
    }}
    // The two narrow ridges belong entirely above the lid datum.
    for [a,b] in [[w,w+(e-w)*0.065],[e-(e-w)*0.065,e]] {
        let first=mesh.positions.len();
        architecture_box(mesh,[a,b,height,height+0.6*scale,n,s],[1.;3]);
        mesh.uvs[first..].fill(sample(1,1));
    }
}

/// Reserve missing furniture against legacy extrusion, displaying its exact
/// source atlas cells. This makes missing coverage visible without changing
/// actor footing, gameplay state or claiming an authored model label.
fn append_missing_source(mesh:&mut TerrainMeshData,g:&GridGeometry,p:&Placement,claimed:&mut [bool]) {
    for i in p.indices(g.width) {
        append_top(&mut mesh.textured,g.bounds(i%g.width,i/g.width).into(),0.,g.uv(i%g.width,i/g.width));
        claimed[i]=true;
        mesh.authored_cells[i]=None;
    }
}

fn append_store_seat(mesh:&mut SurfaceMeshData,b:[f32;4],scale:f32) {
    let [left,right,north,south]=b;
    let cx=(left+right)*0.5;let cz=(north+south)*0.5;
    let hx=(right-left)*0.425;let hz=(south-north)*0.425;
    let [w,e,n,s]=[cx-hx,cx+hx,cz-hz,cz+hz];
    let first=mesh.colors.len();
    model(ModelKind::Cushion).append_fitted(mesh,[w,e,n,s],5.0*scale,2.0*scale);
    // Named upholstery primitives own their paint and weave; never infer a
    // material from its color. Unknown primitives retain authored placement.
    model(ModelKind::Cushion).apply_room_palette(mesh, first);
    model(ModelKind::Cushion).apply_room_finish(mesh, first);
    let x=(e-w)*0.10;let z=(s-n)*0.12;
    architecture_box(mesh,[w+x,e-x,4.3*scale,5.0*scale,n+z,s-z],[0.28,0.29,0.30]);
    for cx in [w+x,e-x-x*0.65] {for cz in [n+z,s-z-z*0.65] {
        architecture_box(mesh,[cx,cx+x*0.65,0.0,4.3*scale,cz,cz+z*0.65],[0.32,0.34,0.35]);
    }}
}

fn append_native_store_seat(mesh:&mut SurfaceMeshData,b:[f32;4],scale:f32,swatches:[[f32;2];3]) {
    let mut upholstery=SurfaceMeshData::default();
    let [left,right,north,south]=b;
    let cx=(left+right)*0.5;let cz=(north+south)*0.5;
    let hx=(right-left)*0.425;let hz=(south-north)*0.425;
    assert!(model(ModelKind::Cushion).append_native_gift_fixture(&mut upholstery,
        [cx-hx,cx+hx,cz-hz,cz+hz],2.5*scale,4.5*scale,
        [swatches[0],swatches[1],swatches[0],swatches[2]]));
    // The source seats are plain padded blocks. The shared floor-cushion
    // model's small central button is separate from its broad cushion faces
    // and perimeter piping; omit only that unsupported central component.
    let triangles=upholstery.indices.chunks_exact(3).filter(|tri| !tri.iter().all(|&i| {
        let position=upholstery.positions[i as usize];
        (position[0]-cx).abs()<hx*0.15&&(position[2]-cz).abs()<hz*0.15
            &&position[1]>6.*scale
    })).map(|tri|[tri[0],tri[1],tri[2]]).collect::<Vec<_>>();
    let top=triangles.iter().flatten().map(|&i|upholstery.positions[i as usize][1])
        .fold(2.5*scale,f32::max);
    let vertical_scale=4.5*scale/(top-2.5*scale);
    let mut remap=vec![None;upholstery.positions.len()];
    for tri in triangles {for i in tri {
        let vertex=*remap[i as usize].get_or_insert_with(|| {
            let vertex=mesh.positions.len()as u32;let mut position=upholstery.positions[i as usize];
            position[1]=2.5*scale+(position[1]-2.5*scale)*vertical_scale;
            let [nx,ny,nz]=upholstery.normals[i as usize];
            let normal=Vec3::new(nx,ny/vertical_scale,nz).normalize();
            mesh.positions.push(position);mesh.normals.push(normal.to_array());
            mesh.uvs.push(upholstery.uvs[i as usize]);
            mesh.colors.push(crate::interior_models::authored_face_color([1.;4],normal));
            vertex
        });mesh.indices.push(vertex);
    }}
    let [w,e,n,s]=[cx-hx,cx+hx,cz-hz,cz+hz];
    let x=(e-w)*0.10;let z=(s-n)*0.12;
    let first=mesh.positions.len();
    architecture_box(mesh,[w+x,e-x,2.*scale,2.5*scale,n+z,s-z],[1.;3]);
    for cx in [w+x,e-x-x*0.65] {for cz in [n+z,s-z-z*0.65] {
        architecture_box(mesh,[cx,cx+x*0.65,0.,2.*scale,cz,cz+z*0.65],[1.;3]);
    }}
    for i in first..mesh.positions.len() {
        mesh.uvs[i]=swatches[2];
        mesh.colors[i]=crate::interior_models::authored_face_color([1.;4],
            Vec3::from_array(mesh.normals[i]));
    }
}

fn append_side_arcade(mesh: &mut SurfaceMeshData, bounds: [f32; 4], base: f32, rise: f32, facing: f32) {
    let [west, east, north, south] = bounds;
    let width = east - west;
    let depth = south - north;
    let center = [(west + east) * 0.5, (north + south) * 0.5];
    let first = mesh.positions.len();
    // Fit before rotating so rectangular source cells keep their footprint.
    model(ModelKind::ArcadeMachine).append_fitted(mesh,
        [-depth * 0.5, depth * 0.5, -width * 0.5, width * 0.5], base, rise);
    model(ModelKind::ArcadeMachine).apply_arcade_finish(mesh, first);
    for p in &mut mesh.positions[first..] {
        let [x, y, z] = *p;
        *p = [center[0] + facing * z, y, center[1] - facing * x];
    }
    for (n, color) in mesh.normals[first..].iter_mut().zip(&mut mesh.colors[first..]) {
        let [x, y, z] = *n;
        let old_shade = crate::interior_models::authored_face_color([1.0;4], Vec3::from_array(*n))[0];
        *n = [facing * z, y, -facing * x];
        let new_shade = crate::interior_models::authored_face_color([1.0;4], Vec3::from_array(*n))[0];
        for channel in &mut color[..3] { *channel *= new_shade / old_shade; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{UVec2, Vec2};
    use std::sync::Arc;
    #[test]
    fn seating_candidate_guard_matches_full_resolver_for_authored_and_edited_profiles() {
        let bits=|seats:Vec<crate::seating::Seat>| seats.into_iter().map(|seat|
            (seat.rect.map(f32::to_bits),seat.height.to_bits(),seat.yaw.map(f32::to_bits)))
            .collect::<Vec<_>>();
        let mut checked=0;
        for object in built_in_profiles().objects.iter().filter(|object|
            profile_kind(object).is_some_and(|kind| matches!(kind,ModelKind::Stool
                |ModelKind::ArcadeStool|ModelKind::Chair|ModelKind::LinkSeat
                |ModelKind::StationBench|ModelKind::Cushion))) {
            let (tiles,g)=fixture(object);
            let mut frame=VisualWorldFrame::default();
            frame.map_id=object.map.as_deref()
                .or_else(||object.maps.as_ref().and_then(|maps|maps.first().map(String::as_str)))
                .unwrap_or(match object.tileset.as_str() {
                    "lab"=>"ElmsLab","pokecenter"=>"UnreviewedPokecenter1F",
                    "train_station"=>"GoldenrodMagnetTrainStation","game_corner"=>"GoldenrodGameCorner",
                    "mart"=>"GoldenrodDeptStore5F","traditional_house"=>"EcruteakItemfinderHouse",
                    _=>"UnreviewedSeatFixture",
                }).into();
            frame.grid_size=UVec2::new(g.width as u32,g.height as u32);
            frame.tile_size=Vec2::splat(8.);frame.tiles=tiles;
            let refs=frame.tiles.iter().collect::<Vec<_>>();
            assert!(has_seating_source_candidate(&frame.map_id,&refs,None),"{}",object.name);
            let guarded=bits(super::super::seating_surfaces_with_candidate_guard(&frame,None,true));
            let full=bits(super::super::seating_surfaces_with_candidate_guard(&frame,None,false));
            assert_eq!(guarded,full,"{}",object.name);
            // Edited live selectors retain the full ownership resolver; they
            // can invalidate canonical chairs or claim overlapping drawings.
            let mut edited=object.clone();edited.tiles[0][0]^=0x80;
            let live=Document {objects:vec![edited],atmosphere:None};
            assert!(has_seating_source_candidate(&frame.map_id,&refs,Some(&live)));
            assert_eq!(bits(super::super::seating_surfaces_with_candidate_guard(&frame,Some(&live),true)),
                bits(super::super::seating_surfaces_with_candidate_guard(&frame,Some(&live),false)),
                "edited {}",object.name);
            // Matching metatile/tile numbers in an outdoor atlas never prove
            // a furniture drawing. This tests the useful skip against the old
            // exhaustive path, including live-profile ownership inputs.
            for tile in &mut frame.tiles {tile.source.tileset_id="johto".into();}
            let refs=frame.tiles.iter().collect::<Vec<_>>();
            assert!(!has_seating_source_candidate(&frame.map_id,&refs,Some(&live)));
            let full=bits(super::super::seating_surfaces_with_candidate_guard(&frame,Some(&live),false));
            assert!(full.is_empty());
            assert_eq!(bits(super::super::seating_surfaces_with_candidate_guard(&frame,Some(&live),true)),full);
            checked+=1;
        }
        assert!(checked>=16,"all authored seating profile families must be covered");
    }

    #[test]
    fn kanto_native_large_tables_use_external_whole_drawings_and_keep_centerpiece_and_footing() {
        let path=std::path::PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .unwrap_or_else(||"content-packs/core-modular.browser.crystalpack".into()));
        if !path.exists() {return;}
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let layout=&loaded.pack().runtime_files()["data/tilesets/house_metatiles.bin"];
        let bitmap=image::load_from_memory(&loaded.pack().runtime_files()["gfx/tilesets/house.png"]).unwrap().to_luma8();
        let palette=&loaded.pack().data().tilesets["house"].palette_map;
        let colors=std::str::from_utf8(&loaded.pack().runtime_files()["gfx/tilesets/house.pal"]).unwrap()
            .lines().filter_map(|line|line.trim().strip_prefix("RGB "))
            .map(|line|line.split(',').map(|v|v.trim().parse::<u8>().unwrap()).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        for map in ["PokemonFanClub","SafariZoneWardensHome","TrainerHouse1F"] {
            let module=&loaded.pack().data().maps[map];
            let w=module.attributes.width as usize*4;let h=module.attributes.height as usize*4;
            let tiles=(0..w*h).map(|i| {let x=i%w;let y=i/w;
                let block=module.blocks[(y/4)*(w/4)+x/4]as u16;
                VisualTile{column:x as u32,row:y as u32,texture:Handle::default(),animation_frames:None,priority:false,
                    source:VisualTileSource{tileset_id:"house".into(),metatile_id:block,
                        subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,
                        tile_index:layout[block as usize*16+(y%4)*4+x%4]as u16}}
            }).collect::<Vec<_>>();
            let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let refs=tiles.iter().collect::<Vec<_>>();
            let p=resolve(map,&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::DiningTable && p.width==8).unwrap();
            assert!(native_large_house_table(map,&p,&refs,&g));
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
            let mut claims=vec![false;tiles.len()];
            assert!(append(&mut mesh,map,&refs,&g,&p,&mut claims,[0,0]));
            assert_eq!(claims.iter().filter(|&&v|v).count(),64);
            assert_eq!(mesh.footing_heights,vec![1.25;tiles.len()]);
            assert!(!native_large_house_table("UnreviewedHouse",&p,&refs,&g));
            assert_native_tabletop_projection(&mesh.textured,&g,&p);
            let decoration=mesh.textured.indices.iter().map(|&i|i as usize)
                .filter(|&i|mesh.textured.positions[i][1]>p.height_pixels+0.001).collect::<Vec<_>>();
            assert!(!decoration.is_empty());
            assert!(decoration.iter().all(|&i|mesh.textured.cutaway_ranges[0].contains(&i)));
            assert!(decoration.iter().all(|&i|mesh.textured.positions[i][1]<=p.height_pixels+18.));
            for (pixel,gray) in native_table_centerpiece::PIXELS.into_iter()
                .zip([255,170,85,0,170,255,85]) {
                let raw=refs[(p.row+pixel[1]/8)*w+p.column+pixel[0]/8].source.tile_index as usize;
                assert_eq!(palette[raw],5,"centerpiece owns its native brown palette and bank");
                assert_eq!(bitmap.get_pixel((raw%16*8+pixel[0]%8)as u32,
                    (raw/16*8+pixel[1]%8)as u32).0[0],gray);
                assert_eq!(colors[20+usize::from((255-gray)/85)],
                    [[26,24,17],[21,17,7],[16,13,3],[7,7,7]][usize::from((255-gray)/85)]);
            }
            for i in p.indices(w) {let mut changed=tiles.clone();changed[i].source.tile_index^=0x80;
                assert!(!resolve(map,&changed.iter().collect::<Vec<_>>(),&g,None).iter()
                    .any(|p|p.kind==ModelKind::DiningTable && p.width==8));}
        }
    }
    #[test]
    fn kanto_native_small_display_stands_keep_four_independent_claims_and_compact_height() {
        for name in ["Shared house flower stand 30 0","Shared house flower stand 30 2",
            "Shared house flower stand 31 0","Shared house flower stand 31 2"] {
            let object=profile(name);let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let p=resolve("MrFujisHouse",&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::LowTable).unwrap();
            assert!(native_small_display_source("MrFujisHouse",&p,&refs,&g));
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
            let mut claims=vec![false;tiles.len()];
            assert!(append(&mut mesh,"MrFujisHouse",&refs,&g,&p,&mut claims,[0,0]));
            assert_eq!(claims.iter().filter(|&&v|v).count(),4);
            assert_eq!(mesh.footing_heights,vec![1.25;tiles.len()]);
            assert!(mesh.textured.positions.iter().all(|p|p[1]<=14.001));
            assert!(resolve("UnreviewedHouse",&refs,&g,None).iter().any(|p|p.kind==ModelKind::FlowerStand));
        }
    }
    #[test]
    fn kanto_native_center_counters_keep_blue_fronts_markings_claims_and_footing() {
        for (block,top) in [(5,vec![0x34;4]),(6,vec![0x0c,0x0c,0x34,0x34]),
            (7,vec![0x34,0x0c]),(0x38,vec![0x34;2])] {
            let map=if block==0x38 {"CinnabarPokecenter2FBeta"}else{"CinnabarPokecenter1F"};
            let mut object=profile("Shared lab computer workstation");
            object.tileset="pokecenter".into();object.metatile=block;object.origin=[0,0];
            object.metatiles=None;object.tiles=vec![top.clone(),vec![0x24;top.len()]];object.ground=0x11;
            let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let p=resolve(map,&refs,&g,None).into_iter()
                .find(|p|p.kind==ModelKind::ReceptionCounter).unwrap_or_else(||panic!("missing counter block {block:02x}"));
            assert!(native_center_counter_source(map,&p,&refs,&g).is_some());
            assert!(native_center_counter_source("UnreviewedPokecenter1F",&p,&refs,&g).is_none());
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            assert!(append(&mut mesh,map,&refs,&g,&p,&mut claimed,[0,0]));
            assert!(mesh.solid.indices.is_empty());assert!(!mesh.textured.indices.is_empty());
            assert_eq!(mesh.footing_heights,vec![1.25;tiles.len()]);
            assert_eq!(claimed.iter().filter(|&&v|v).count(),top.len()*2);
            for x in 0..top.len() {if top[x]==0x0c {
                let uv=g.uv(p.column+x,p.row);
                assert!(mesh.textured.uvs.contains(&[uv.0,uv.2]));
                assert!(mesh.textured.uvs.contains(&[uv.1,uv.3]));
            }}
            for i in p.indices(g.width) {let mut changed=tiles.clone();changed[i].source.tile_index^=0x80;
                assert!(!resolve(map,&changed.iter().collect::<Vec<_>>(),&g,None)
                    .iter().any(|q|q.kind==ModelKind::ReceptionCounter));
            }
        }
    }
    #[test]
    fn kanto_native_mansion_desks_and_seats_keep_complete_drawings_claims_and_footing() {
        let desk=vec![vec![0x2a,0x2b,0x2c,0x2d],vec![0x3a,0x3b,0x3c,0x3d],
            vec![0x4a,0x4b,0x4c,0x4d],vec![0x5a,0x5b,0x11,0x11]];
        for (block,origin,blocks,drawing,kind) in [
            (0x12,[0,2],Some(vec![vec![0x12],vec![0x13]]),desk.clone(),ModelKind::LabWorkstation),
            (0x18,[2,2],Some(vec![vec![0x18,0x19],vec![0x1b,0x1a]]),desk.clone(),ModelKind::LabWorkstation),
            (0x1c,[0,0],None,desk,ModelKind::LabWorkstation),
        ] {
            let mut object=profile("Shared lab computer workstation");
            object.tileset="mansion".into();object.metatile=block;object.origin=origin;
            object.metatiles=blocks;object.tiles=drawing;object.ground=0x11;
            let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let p=resolve("CeladonMansion3F",&refs,&g,None).into_iter().find(|p|p.kind==kind).unwrap();
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            assert!(append(&mut mesh,"CeladonMansion3F",&refs,&g,&p,&mut claimed,[0,0]));
            assert!(mesh.solid.indices.is_empty());assert!(!mesh.textured.indices.is_empty());
            assert_eq!(claimed.iter().filter(|&&v|v).count(),p.width*p.height);
            assert_eq!(mesh.footing_heights,vec![1.25;tiles.len()]);
            assert!(!resolve("CeladonMansion3F",&refs,&g,None).iter().any(|q|q.kind==ModelKind::LinkSeat));
            for i in p.indices(g.width) {
                let mut changed=tiles.clone();changed[i].source.tile_index^=0x80;
                assert!(!resolve("CeladonMansion3F",&changed.iter().collect::<Vec<_>>(),&g,None)
                    .iter().any(|q|q.kind==kind));
                let mut changed=tiles.clone();changed[i].source.subtile_column^=1;
                assert!(!resolve("CeladonMansion3F",&changed.iter().collect::<Vec<_>>(),&g,None)
                    .iter().any(|q|q.kind==kind));
            }
            assert!(!resolve("CeladonMansion1F",&refs,&g,None).iter().any(|q|q.kind==kind));
        }
    }
    #[test]
    fn kanto_native_upstairs_seats_bind_whole_yellow_art_without_changing_johto() {
        for name in ["Pokecenter upstairs seat 2d 0","Pokecenter upstairs seat 2d 2","Pokecenter upstairs seat 29 0"] {
            let object=profile(name);let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let p=resolve("CinnabarPokecenter2FBeta",&refs,&g,None).into_iter()
                .find(|p|p.kind==ModelKind::LinkSeat).unwrap();
            assert!(native_link_seat_source("CinnabarPokecenter2FBeta",&p,&refs,&g).is_some());
            assert!(native_link_seat_source("GoldenrodPokecenter2F",&p,&refs,&g).is_none());
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            assert!(append(&mut mesh,"CinnabarPokecenter2FBeta",&refs,&g,&p,&mut claimed,[0,0]));
            assert!(mesh.solid.indices.is_empty());assert_eq!(claimed.iter().filter(|&&v|v).count(),4);
            assert_eq!(mesh.footing_heights,vec![1.25;tiles.len()]);
            let mut johto=TerrainMeshData::default();
            assert!(append(&mut johto,"GoldenrodPokecenter2F",&refs,&g,&p,&mut vec![false;tiles.len()],[0,0]));
            assert!(!johto.solid.indices.is_empty());
            for i in p.indices(g.width) {let mut changed=tiles.clone();changed[i].source.tile_index^=0x80;
                assert!(!resolve("CinnabarPokecenter2FBeta",&changed.iter().collect::<Vec<_>>(),&g,None)
                    .iter().any(|q|q.kind==ModelKind::LinkSeat));
            }
        }
    }
    #[test]
    fn native_gate_plants_cover_every_external_pack_crown_including_mirrored_planters() {
        // Validate the production catalog even when the optional pack is absent.
        Document::parse(include_bytes!("../../../../modpacks/voxel-view/profiles.json")).unwrap();
        let explicit=std::env::var_os("CRYSTAL_RENDER_TEST_PACK");
        let path=explicit.as_ref().map(std::path::PathBuf::from).unwrap_or_else(||
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        assert!(explicit.is_none()||path.is_file(),"explicit external pack must exist");
        if !path.is_file(){return;}
        let pack=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let layout=&pack.pack().runtime_files()["data/tilesets/gate_metatiles.bin"];
        let mut total=0;let mut mirrored=0;
        for (map,module) in &pack.pack().data().maps {
            if !(super::super::gate_counters::reviewed_gate_map(map)||map=="GoldenrodUnderground"){continue;}
            let (w,h)=(module.attributes.width as usize*4,module.attributes.height as usize*4);
            let mut tiles=Vec::new();
            for y in 0..h {for x in 0..w {
                let block=module.blocks[y/4*(w/4)+x/4];
                tiles.push(VisualTile{animation_frames:None,column:x as u32,row:y as u32,texture:Default::default(),priority:false,
                    source:VisualTileSource{tileset_id:Arc::from("gate"),metatile_id:block,subtile_column:(x%4)as u8,
                        subtile_row:(y%4)as u8,tile_index:layout[block as usize*16+y%4*4+x%4]as u16}});
            }}
            let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let refs=tiles.iter().collect::<Vec<_>>();
            let expected=tiles.iter().filter(|t|t.source.tile_index==0x38).count();
            let plants=resolve(map,&refs,&g,None).into_iter().filter(|p|native_gate_plant_source(map,p,&refs,&g)).collect::<Vec<_>>();
            assert_eq!(plants.len(),expected,"{map}: every source crown must be bound");
            for p in plants {
                total+=1;
                if tiles[p.row*w+p.column].source.metatile_id==0x3a {mirrored+=1;}
                let mut mesh=TerrainMeshData{footing_heights:vec![2.75;tiles.len()],..Default::default()};
                let mut claimed=vec![false;tiles.len()];assert!(append(&mut mesh,map,&refs,&g,&p,&mut claimed,[0,0]));
                assert!(mesh.solid.positions.is_empty());
                assert_eq!(claimed.iter().filter(|&&v|v).count(),p.width*p.height);
                assert!(mesh.footing_heights.iter().all(|&v|v==2.75));
            }
        }
        assert_eq!(total,52);assert_eq!(mirrored,3);
    }

    #[test]
    fn native_gate_plants_keep_green_crowns_gold_pots_and_exact_source_claims() {
        for name in ["Shared gate planter 0","Shared gate planter 2","Shared gate corridor planter","Shared gate long planter","Shared gate mirrored long planter"] {
            let object=if name=="Shared gate mirrored long planter" {
                let mut o=profile("Shared gate long planter");o.name=name.into();o.metatile=0x3a;o.origin=[2,0];o
            } else {profile(name)};
            let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let map=if name.contains("corridor")||name.contains("long") {"GoldenrodUnderground"}else{"Route31VioletGate"};
            let p=resolve(map,&refs,&g,None).into_iter().find(|p|matches!(p.kind,ModelKind::PottedPlant|ModelKind::LongPlanter)).unwrap();
            assert!(native_gate_plant_source(map,&p,&refs,&g),"{name}");
            assert!(!native_gate_plant_source("UnreviewedGate",&p,&refs,&g));
            let mut raised=p.clone();raised.base_pixels=4.;
            assert!(!native_gate_plant_source(map,&raised,&refs,&g));
            let mut mesh=TerrainMeshData{footing_heights:vec![2.75;tiles.len()],..Default::default()};
            let mut claims=vec![false;tiles.len()];
            assert!(append(&mut mesh,map,&refs,&g,&p,&mut claims,[0,0]));
            assert!(mesh.solid.indices.is_empty());assert_eq!(claims.iter().filter(|&&v|v).count(),p.width*p.height);
            assert!(mesh.footing_heights.iter().all(|&v|v==2.75));
            let top=if p.kind==ModelKind::PottedPlant {16.}else{17.};
            assert!(mesh.textured.positions.iter().all(|v|v[1]<=top+1e-4));
            let pot_y=(p.height-1)*8;
            for uv in native_source_swatches(&p,&g,[[3,1],[6,1],[3,pot_y+1],[1,pot_y],[3,pot_y]]) {
                assert!(mesh.textured.uvs.contains(&uv));
            }
            for i in p.indices(g.width) {for field in 0..4 {
                let mut changed=tiles.clone();let source=&mut changed[i].source;
                match field {0=>source.tile_index^=128,1=>source.metatile_id^=128,
                    2=>source.subtile_column^=1,_=>source.subtile_row^=1}
                assert!(!native_gate_plant_source(map,&p,&changed.iter().collect::<Vec<_>>(),&g),"{name}: cell{i} field{field}");
            }}
            let mut changed=object.clone();changed.top_pixels+=1;
            let active=Document{objects:vec![changed],atmosphere:None};
            assert!(!resolve(map,&refs,&g,Some(&active)).iter().any(|p|native_gate_plant_source(map,p,&refs,&g)),"{name}: live custom owner");
        }
    }

    #[test]
    fn kanto_basement_plants_keep_source_gold_pots_and_complete_art_ownership() {
        for name in ["Shared facility potted plant 0d","Shared facility potted plant 0e"] {
            let object=profile(name);let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let p=resolve("TrainerHouseB1F",&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::PottedPlant).unwrap();
            assert!(native_facility_plant("TrainerHouseB1F",&p,&refs,&g));
            assert!(native_facility_plant("SilphCo1F",&p,&refs,&g));
            assert!(!native_facility_plant("TeamRocketBaseB2F",&p,&refs,&g));
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            assert!(append(&mut mesh,"TrainerHouseB1F",&refs,&g,&p,&mut claimed,[0,0]));
            assert!(mesh.solid.indices.is_empty());assert_eq!(claimed.iter().filter(|&&x|x).count(),8);
            assert_eq!(mesh.footing_heights,vec![1.25;tiles.len()]);
            let swatches=native_source_swatches(&p,&g,[[2,2],[2,7],[6,4],[3,25],[4,25],[6,16]]);
            assert!(swatches.iter().all(|uv|mesh.textured.uvs.contains(uv)));
            for i in p.indices(g.width) {let mut changed=tiles.clone();changed[i].source.tile_index^=128;
                assert!(!native_facility_plant("TrainerHouseB1F",&p,&changed.iter().collect::<Vec<_>>(),&g));}
        }
    }
    #[test]
    fn kanto_native_arcade_rows_keep_whole_claims_footing_and_reject_partial_fallbacks() {
        for map in ["CeladonGameCorner","GoldenrodGameCorner"] {for name in ["Shared game row cabinet 7 0","Shared game row cabinet 7 2",
            "Shared game row cabinet 11 0","Shared game corner rear machine 0","Shared game corner rear machine 2"] {
            let object=profile(name);let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let p=resolve(map,&refs,&g,None).into_iter()
                .find(|p|matches!(p.kind,ModelKind::ArcadeMachine|ModelKind::ArcadePair)).unwrap();
            assert!(native_arcade_source(map,&p,&refs,&g).is_some());
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            assert!(append(&mut mesh,map,&refs,&g,&p,&mut claimed,[0,0]));
            assert!(mesh.solid.indices.is_empty());assert_eq!(claimed.iter().filter(|&&v|v).count(),p.width*p.height);
            assert_eq!(mesh.footing_heights,vec![1.25;tiles.len()]);
            for i in p.indices(g.width) {let mut changed=tiles.clone();changed[i].source.tile_index^=0x40;
                assert!(!resolve(map,&changed.iter().collect::<Vec<_>>(),&g,None).iter()
                    .any(|p|matches!(p.kind,ModelKind::ArcadeMachine|ModelKind::ArcadePair)));
            }
            let mut johto=TerrainMeshData::default();
            assert!(append(&mut johto,"UnreviewedArcadeRoom",&refs,&g,&p,&mut vec![false;tiles.len()],[0,0]));
            assert!(!johto.solid.indices.is_empty());
        }}
    }
    #[test]
    fn kanto_native_mart_counters_keep_complete_source_paint_claims_and_footing() {
        for (block,origin,top,front) in [(0x0c,[2,0],vec![0x2e,0x1e],vec![0x18,0x1a]),
            (0x0d,[0,0],vec![0x1e;4],vec![0x1a;4]),(0x0e,[0,0],vec![0x1e,0x2f],vec![0x1a,0x19]),
            (0x22,[0,0],vec![0x20,0x21],vec![0x30,0x31])] {
            let mut object=profile("Shared facility paired bookcases 06");object.map=None;object.maps=None;
            object.parts.clear();object.metatiles=None;object.tileset="mart".into();object.metatile=block;
            object.origin=origin;object.ground=0x48;object.tiles=vec![top,front];
            let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let p=resolve("CeruleanMart",&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::ReceptionCounter).unwrap();
            let mut mesh=TerrainMeshData {footing_heights:vec![1.25;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];assert!(append(&mut mesh,"CeruleanMart",&refs,&g,&p,&mut claimed,[0,0]));
            assert!(mesh.solid.indices.is_empty());assert_eq!(claimed.iter().filter(|&&v|v).count(),p.width*2);
            assert_eq!(mesh.footing_heights,vec![1.25;tiles.len()]);
            for i in p.indices(g.width) {
                let mut changed=tiles.clone();changed[i].source.tile_index^=0x80;
                assert!(!resolve("CeruleanMart",&changed.iter().collect::<Vec<_>>(),&g,None).iter().any(|p|p.kind==ModelKind::ReceptionCounter));
            }
            for map in ["AzaleaMart","BlackthornMart","CherrygroveMart","EcruteakMart","OlivineMart","VioletMart"] {
                let mut johto=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
                let mut claims=vec![false;tiles.len()];
                assert!(append(&mut johto,map,&refs,&g,&p,&mut claims,[0,0]));
                assert!(johto.solid.indices.is_empty());
                assert_eq!(claims.iter().filter(|&&v|v).count(),p.width*2);
                assert_eq!(johto.footing_heights,vec![1.25;tiles.len()]);
            }
            let mut unrelated=TerrainMeshData::default();
            assert!(append(&mut unrelated,"UnreviewedJohtoMart",&refs,&g,&p,&mut vec![false;tiles.len()],[0,0]));
            assert!(!unrelated.solid.indices.is_empty());
        }
    }
    #[test]
    fn kanto_native_saffron_column_chairs_use_source_paint_and_keep_footing() {
        let mut object=profile("Shared facility paired bookcases 06");
        object.map=None;object.maps=None;object.parts.clear();object.metatiles=None;
        object.tileset="train_station".into();object.metatile=2;object.origin=[0,0];object.ground=0x3d;
        object.tiles=vec![vec![0x40,0x41],vec![0x42,0x43],vec![0x40,0x41],vec![0x42,0x43]];
        let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        let seats=resolve("SaffronMagnetTrainStation",&refs,&g,None).into_iter()
            .filter(|p|p.kind==ModelKind::Chair).collect::<Vec<_>>();assert_eq!(seats.len(),2);
        let mut mesh=TerrainMeshData {footing_heights:vec![2.5;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        for seat in &seats {
            assert!(native_station_chair_source("SaffronMagnetTrainStation",seat,&refs,&g));
            assert!(append(&mut mesh,"SaffronMagnetTrainStation",&refs,&g,seat,&mut claimed,[0,0]));
        }
        assert_eq!(claimed.iter().filter(|&&v|v).count(),8);assert!(mesh.solid.indices.is_empty());
        assert_eq!(mesh.footing_heights,vec![2.5;tiles.len()]);
        for seat in &seats {for i in seat.indices(g.width) {
            let mut changed=tiles.clone();changed[i].source.tile_index^=1;
            assert!(!resolve("SaffronMagnetTrainStation",&changed.iter().collect::<Vec<_>>(),&g,None).iter()
                .any(|p|p.kind==ModelKind::Chair && p.row==seat.row));
        }}
    }
    #[test]
    fn kanto_native_station_ticket_bank_is_four_upright_boxes_with_unchanged_footing() {
        let mut object=profile("Shared facility paired bookcases 06");
        object.map=None;object.maps=None;object.parts.clear();object.tileset="train_station".into();
        object.metatiles=Some(vec![vec![0x2c],vec![0x2d]]);object.origin=[2,0];object.ground=0x3d;
        object.tiles=(0..8).map(|y|match y {6=>vec![0x46,0x82],7=>vec![0x47,0x83],
            _=>if y%2==0 {vec![0x44,0x80]}else{vec![0x45,0x81]}}).collect();
        let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        let p=resolve("GoldenrodMagnetTrainStation",&refs,&g,None).into_iter()
            .find(|p|native_station_ticket_source("GoldenrodMagnetTrainStation",p,&refs,&g)).unwrap();
        let mut mesh=TerrainMeshData {footing_heights:vec![2.5;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        assert!(append(&mut mesh,"GoldenrodMagnetTrainStation",&refs,&g,&p,&mut claimed,[0,0]));
        assert_eq!(claimed.iter().filter(|&&v|v).count(),16);
        assert_eq!(mesh.footing_heights,vec![2.5;tiles.len()]);assert!(mesh.solid.indices.is_empty());
        for unit in 0..4 {
            assert!(mesh.textured.positions.iter().any(|v|v[1]>19.
                && v[2]>unit as f32*16. && v[2]<(unit+1)as f32*16.));
        }
        assert!(mesh.textured.normals.iter().any(|v|v[0]< -0.99));
        for i in p.indices(g.width) {
            let mut changed=tiles.clone();changed[i].source.tile_index^=1;
            assert!(!resolve("GoldenrodMagnetTrainStation",&changed.iter().collect::<Vec<_>>(),&g,None)
                .iter().any(|p|p.kind==ModelKind::VendingMachine));
        }
        assert!(!resolve("SaffronMagnetTrainStation",&refs,&g,None).iter().any(|p|p.kind==ModelKind::VendingMachine));
    }
    #[test]
    fn kanto_native_station_planters_own_every_complete_variant_and_keep_footing() {
        for (block,column,height) in [(0x0c,0,4),(0x0e,2,4),(0x20,0,4),(0x20,2,4),
            (0x2f,0,2),(0x2f,2,2),(0x33,0,3)] {
            let mut object=profile("Shared facility paired bookcases 06");
            object.map=None;object.maps=None;object.parts.clear();object.metatiles=None;
            object.tileset="train_station".into();object.metatile=block;
            object.origin=[column,0];object.ground=0x3d;
            object.tiles=(0..height).map(|y|if y==0 {vec![0x50,0x51]}
                else if y==height-1 {vec![0x54,0x55]}else{vec![0x52,0x53]}).collect();
            let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let p=resolve("SaffronMagnetTrainStation",&refs,&g,None).into_iter()
                .find(|p|p.kind==ModelKind::LongPlanter).unwrap();
            let mut mesh=TerrainMeshData {footing_heights:vec![2.;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            assert!(append(&mut mesh,"SaffronMagnetTrainStation",&refs,&g,&p,&mut claimed,[0,0]));
            assert_eq!(claimed.iter().filter(|&&v|v).count(),2*height);
            assert_eq!(mesh.footing_heights,vec![2.;tiles.len()]);
            assert!(mesh.solid.positions.is_empty());
            assert!(mesh.textured.positions.len()>2*height*4);
            for i in p.indices(g.width) {
                let mut changed=tiles.clone();changed[i].source.tile_index^=0x80;
                assert!(!resolve("SaffronMagnetTrainStation",&changed.iter().collect::<Vec<_>>(),&g,None)
                    .iter().any(|p|p.kind==ModelKind::LongPlanter));
            }
            assert!(!resolve("ViridianGym",&refs,&g,None).iter().any(|p|p.kind==ModelKind::LongPlanter));
        }
    }
    #[test]
    fn kanto_native_station_wall_and_rail_are_complete_closed_volumes_over_unchanged_ground() {
        for (block,origin,width,height,kind) in [(0x0a,[0,0],4,8,ModelKind::WallClinical),
            (0x16,[0,0],4,8,ModelKind::WallClinical),(6,[0,2],4,2,ModelKind::WallPanel),
            (8,[2,2],2,2,ModelKind::WallPanel)] {
            let mut object=profile("Shared facility paired bookcases 06");
            object.map=None;object.maps=None;object.parts.clear();object.metatiles=None;
            object.tileset="train_station".into();object.metatile=block;object.origin=origin;object.ground=0x3d;
            if height==2 {object.ground=0x3e;}
            object.tiles=(0..height).map(|y|if height==2 {vec![if y==0 {8}else{0x18};width]}
                else if y<4 {if block==0x0a {vec![0x37,0x38,0x30,0x30]}else{vec![0x30;4]}}
                else {vec![if y==7 {0x33}else{0x3e};4]}).collect();
            if height==8 {object.metatiles=Some(vec![vec![block],vec![0x0b]]);}
            let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let p=resolve("SaffronMagnetTrainStation",&refs,&g,None).into_iter().find(|p|p.kind==kind).unwrap();
            let mut mesh=TerrainMeshData {footing_heights:vec![1.25;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            assert!(append(&mut mesh,"SaffronMagnetTrainStation",&refs,&g,&p,&mut claimed,[0,0]));
            assert_eq!(claimed.iter().filter(|&&v|v).count(),width*height);
            assert_eq!(mesh.footing_heights,vec![1.25;tiles.len()]);
            assert!(mesh.textured.positions.iter().any(|v|v[1]==if height==8 {32.}else{11.}));
            if height==8 {
                let south=(p.row+p.height)as f32*g.tile_height;
                assert!(mesh.textured.positions.iter().filter(|v|v[1]==0.)
                    .all(|v|v[2]>=south-g.tile_height),"no false floor behind the rear wall");
                let area=mesh.textured.indices.chunks_exact(3).filter_map(|tri| {
                    let a=tri.iter().map(|&i|Vec3::from_array(mesh.textured.positions[i as usize])).collect::<Vec<_>>();
                    a.iter().all(|v|(v.z-south).abs()<0.001).then(||(a[1]-a[0]).cross(a[2]-a[0]).z*0.5)
                }).sum::<f32>();
                assert!((area-32.*32.).abs()<0.01,"entire closed front faces the station, never culled");
            }
            for i in p.indices(g.width) {
                let mut changed=tiles.clone();changed[i].source.tile_index^=0x80;
                assert!(!resolve("SaffronMagnetTrainStation",&changed.iter().collect::<Vec<_>>(),&g,None)
                    .iter().any(|p|p.kind==kind && p.height==height));
            }
        }
    }
    #[test]
    fn kanto_native_red_desk_requires_complete_top_and_feet_without_changing_support() {
        let mut object=profile("Shared facility paired bookcases 06");
        object.map=None;object.maps=None;object.parts.clear();
        object.tileset="players_house".into();object.metatile=0x20;
        object.metatiles=Some(vec![vec![0x20],vec![0x12]]);object.origin=[0,2];object.ground=1;
        object.tiles=vec![vec![0x23,0x22,0x22,0x24],vec![0x33,0x32,0x32,0x34],vec![0x1c,0x40,0x40,0x1d]];
        let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        let p=resolve("RedsHouse2F",&refs,&g,None).into_iter()
            .find(|p|p.kind==ModelKind::DiningTable && p.height==3).unwrap();
        assert!(native_house_dining_table("RedsHouse2F",&refs,&g,&p));
        let mut mesh=TerrainMeshData {footing_heights:vec![2.25;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        assert!(append(&mut mesh,"RedsHouse2F",&refs,&g,&p,&mut claimed,[0,0]));
        assert_eq!(claimed.iter().filter(|&&v|v).count(),12);
        assert_eq!(mesh.footing_heights,vec![2.25;tiles.len()]);
        assert!(mesh.solid.positions.is_empty());
        let prop=&mesh.textured.positions[48..];
        let extent=|axis:usize|prop.iter().map(|v|v[axis]).fold(f32::NEG_INFINITY,f32::max)
            -prop.iter().map(|v|v[axis]).fold(f32::INFINITY,f32::min);
        assert!((extent(0)-31.28).abs()<0.001 && (extent(2)-16.).abs()<0.001);
        assert!(prop.iter().all(|v|v[1]>=0. && v[1]<=10.));
        assert!(!resolve("PlayersHouse2F",&refs,&g,None).iter()
            .any(|p|p.kind==ModelKind::DiningTable && p.width==4 && p.height==3));
        for i in p.indices(g.width) {
            let mut changed=tiles.clone();changed[i].source.tile_index^=0x80;
            assert!(!resolve("RedsHouse2F",&changed.iter().collect::<Vec<_>>(),&g,None).iter()
                .any(|p|p.kind==ModelKind::DiningTable && p.height==3));
        }
    }
    #[test]
    fn kanto_native_furniture_physical_scale_keeps_console_and_seats_smaller_than_tv() {
        let object=profile("Shared game corner stool 5 0");
        let (tiles,mut g)=fixture(&object);
        let refs=tiles.iter().collect::<Vec<_>>();
        for scale in [0.5,1.,2.] {
            g.tile_width=8.*scale;g.tile_height=8.*scale;
            let dimensions=|kind| {
                let (depth_pixels,height_pixels)=proportions(kind,2,2);
                let p=Placement {kind,column:0,row:0,width:2,height:2,ground:1,
                    depth_pixels,height_pixels,base_pixels:0.,footing_pixels:None};
                let mut mesh=TerrainMeshData {footing_heights:vec![2.75;tiles.len()],..Default::default()};
                let mut claimed=vec![false;tiles.len()];
                assert!(append(&mut mesh,"PlayersHouse2F",&refs,&g,&p,&mut claimed,[0,0]));
                assert_eq!(claimed.iter().filter(|&&v|v).count(),4);
                assert_eq!(mesh.footing_heights,vec![2.75;tiles.len()]);
                // Source underlay still owns all four tiles; measure only the prop.
                let vertices=mesh.solid.positions.iter().chain(mesh.textured.positions.iter().skip(16));
                let mut lo=[f32::INFINITY;3];let mut hi=[f32::NEG_INFINITY;3];
                for v in vertices {for axis in 0..3 {lo[axis]=lo[axis].min(v[axis]);hi[axis]=hi[axis].max(v[axis]);}}
                [hi[0]-lo[0],hi[1]-lo[1],hi[2]-lo[2]]
            };
            let tv=dimensions(ModelKind::Television);
            for kind in [ModelKind::GameConsole,ModelKind::Stool,ModelKind::ArcadeStool,
                ModelKind::Chair,ModelKind::LinkSeat] {
                let prop=dimensions(kind);
                let volume=|d:[f32;3]|d.into_iter().product::<f32>();
                assert!(prop[0]<tv[0] && volume(prop)<volume(tv),
                    "{kind:?}: {prop:?} must be narrower and smaller overall than TV {tv:?}");
            }
            let console=dimensions(ModelKind::GameConsole);
            assert!((console[0]/scale-6.).abs()<0.001);
            assert!((console[1]/scale-3.).abs()<0.001);
            let mut crt=SurfaceMeshData::default();
            assert!(model(ModelKind::Television).append_native_television_screen(
                &mut crt,[0.,15.28*scale,0.,9.*scale],20.*scale,[[0.,0.];5]));
            let extent=|axis:usize| {
                crt.positions.iter().map(|v|v[axis]).fold(f32::NEG_INFINITY,f32::max)
                    -crt.positions.iter().map(|v|v[axis]).fold(f32::INFINITY,f32::min)
            };
            assert!(console.iter().enumerate().all(|(axis,&size)|size<extent(axis)),
                "console must also be smaller than the bedroom's cabinet-free CRT");
            for kind in [ModelKind::Stool,ModelKind::ArcadeStool,ModelKind::Chair,ModelKind::LinkSeat] {
                let seat=dimensions(kind);
                assert!(seat[0]<extent(0) && seat[2]<extent(2),"{kind:?} seat exceeds CRT footprint");
            }
        }
    }
    #[test]
    fn kanto_native_home_fixture_palette_keeps_claims_and_rejects_changed_drawings() {
        for (kind, block, origin, drawing) in [
            (ModelKind::DrawerCabinet, 0x11, [2, 1], vec![vec![0x11, 0x11], vec![0x0e, 0x0f], vec![0x3a, 0x3b]]),
            (ModelKind::GameConsole, 0x13, [2, 2], vec![vec![0x4a, 0x4b], vec![0x5a, 0x5b]]),
        ] {
            let mut object = profile("Shared facility paired bookcases 06");
            object.map = None; object.maps = None; object.parts.clear(); object.metatiles = None;
            object.tileset = "players_house".into(); object.metatile = block;
            object.origin = origin; object.ground = 1; object.tiles = drawing;
            let (mut tiles, g) = fixture(&object);
            let refs = tiles.iter().collect::<Vec<_>>();
            let p = resolve("RedsHouse2F", &refs, &g, None).into_iter()
                .find(|p| p.kind == kind).unwrap();
            assert!(native_home_fixture_source(&p, &refs, &g).is_some());
            let mut mesh = TerrainMeshData { footing_heights: vec![2.75; tiles.len()], ..Default::default() };
            let mut claimed = vec![false; tiles.len()];
            assert!(append(&mut mesh, "RedsHouse2F", &refs, &g, &p, &mut claimed, [0, 0]));
            assert!(mesh.solid.positions.is_empty());
            assert_eq!(mesh.footing_heights, vec![2.75; tiles.len()]);
            assert_eq!(claimed.iter().filter(|&&v| v).count(), p.width * p.height);
            let mut johto = TerrainMeshData::default();
            assert!(append(&mut johto, "ElmsHouse", &refs, &g, &p,
                &mut vec![false; tiles.len()], [0, 0]));
            assert!(!johto.solid.positions.is_empty(), "native paint must not spill into an unreviewed room");
            drop(refs);
            tiles[p.row * g.width + p.column].source.tile_index ^= 0x80;
            let changed = tiles.iter().collect::<Vec<_>>();
            assert!(native_home_fixture_source(&p, &changed, &g).is_none());
            assert!(!resolve("RedsHouse2F", &changed, &g, None).iter().any(|p| p.kind == kind));
        }
    }
    #[test]
    fn kanto_native_cafe_stools_share_rectangular_seats_with_their_own_gray_art() {
        for name in ["Shared game corner stool 5 0","Shared game corner stool 5 2",
            "Shared game corner stool 6 0","Shared game corner stool 6 2"] {
            let object=profile(name);let (mut tiles,g)=fixture(&object);
            let refs=tiles.iter().collect::<Vec<_>>();
            let p=resolve("CeladonCafe",&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::ArcadeStool).unwrap();
            let mut mesh=TerrainMeshData{footing_heights:vec![1.75;tiles.len()],..Default::default()};
            assert!(append(&mut mesh,"CeladonCafe",&refs,&g,&p,&mut vec![false;tiles.len()],[0,0]));
            assert!(mesh.solid.positions.is_empty());
            assert!(mesh.textured.positions.iter().all(|p|p[1]>=0. && p[1]<=5.));
            assert_eq!(mesh.footing_heights,vec![1.75;tiles.len()]);
            let mut arcade=TerrainMeshData::default();
            assert!(append(&mut arcade,"CeladonGameCorner",&refs,&g,&p,&mut vec![false;tiles.len()],[0,0]));
            assert!(!arcade.solid.positions.is_empty());
            drop(refs);tiles[0].source.tile_index^=1;
            assert!(!resolve("CeladonCafe",&tiles.iter().collect::<Vec<_>>(),&g,None)
                .iter().any(|p|p.kind==ModelKind::ArcadeStool));
        }
    }
    #[test]
    fn kanto_native_cafe_front_stools_require_complete_block19_drawings() {
        for column in [0,2] {
            let mut object=profile("Shared game corner stool 5 0");
            object.metatile=0x19;object.metatiles=None;object.origin=[column,0];
            object.tiles=vec![vec![0x0a,0x0b],vec![0x1a,0x1b]];
            object.parts.clear();object.footing_pixels=None;
            let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let p=resolve("CeladonCafe",&refs,&g,None).into_iter()
                .find(|p|p.kind==ModelKind::ArcadeStool).unwrap();
            let mut mesh=TerrainMeshData {footing_heights:vec![2.;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            assert!(append(&mut mesh,"CeladonCafe",&refs,&g,&p,&mut claimed,[0,0]));
            assert_eq!(claimed.iter().filter(|&&v|v).count(),4);
            assert!(mesh.solid.positions.is_empty());
            assert!(mesh.textured.positions.iter().any(|v|v[1]==5.));
            assert_eq!(mesh.footing_heights,vec![2.;tiles.len()]);
            assert!(!resolve("CeladonGameCorner",&refs,&g,None).iter()
                .any(|p|p.kind==ModelKind::ArcadeStool));
            drop(refs);tiles[0].source.tile_index^=0x80;
            assert!(!resolve("CeladonCafe",&tiles.iter().collect::<Vec<_>>(),&g,None).iter()
                .any(|p|p.kind==ModelKind::ArcadeStool));
        }
    }
    #[test]
    fn johto_player_home_appliances_and_bed_bind_only_complete_drawings() {
        for (kind,atlas,block,origin,drawing,map) in [
            (ModelKind::Radio,"players_room",1,[2,2],vec![vec![0x42,0x43],vec![0x52,0x53]],"PlayersHouse2F"),
            (ModelKind::KitchenStove,"players_house",7,[0,2],vec![vec![0x50,0x51],vec![0x52,0x53]],"PlayersHouse1F"),
            (ModelKind::KitchenSink,"players_house",7,[2,2],vec![vec![0x43,0x45],vec![0x18,0x19]],"PlayersHouse1F"),
            (ModelKind::Refrigerator,"players_house",15,[0,1],vec![vec![10,11],vec![26,27],vec![42,43]],"PlayersHouse1F"),
            (ModelKind::BedFeathery,"players_room",0x1b,[0,0],vec![vec![3,4],vec![0x13,0x14],vec![0x23,0x24],vec![0x33,0x34]],"PlayersHouse2F"),
        ] {
            let mut object=profile("Shared lab round stool");object.map=None;object.maps=None;
            object.parts.clear();object.metatiles=None;object.tileset=atlas.into();
            object.metatile=block;object.origin=origin;object.ground=if kind==ModelKind::Radio {2}else{1};object.tiles=drawing;
            let (mut tiles,g)=fixture(&object);
            if atlas=="players_house" {tiles.last_mut().unwrap().source.tile_index=0x11;}
            let refs=tiles.iter().collect::<Vec<_>>();
            let placements=resolve(map,&refs,&g,None);
            let p=placements.iter().find(|p|p.kind==kind).unwrap_or_else(||panic!("missing fixture {kind:?} in {atlas}"));
            assert!(native_home_fixture_source(p,&refs,&g).is_some());
            let mut mesh=TerrainMeshData{authored_cells:vec![None;tiles.len()],
                footing_heights:vec![2.5;tiles.len()],..Default::default()};
            assert!(append(&mut mesh,map,&refs,&g,p,&mut vec![false;tiles.len()],[0,0]));
            assert!(mesh.solid.positions.is_empty(),"generic appliance pigment must not remain");
            assert_eq!(mesh.footing_heights,vec![2.5;tiles.len()]);
            tiles[0].source.tile_index=0xffff;
            assert!(native_home_fixture_source(p,&tiles.iter().collect::<Vec<_>>(),&g).is_none());
        }
    }

    #[test]
    fn johto_player_home_bookcase_keeps_geometry_with_native_gray_books() {
        let mut object=profile("Shared facility paired bookcases 06");
        object.map=None;object.maps=None;object.parts.clear();object.metatiles=None;
        object.tileset="players_room".into();object.metatile=3;object.origin=[2,0];object.ground=2;
        object.tiles=vec![vec![5,6],vec![0x15,0x16],vec![0x25,0x26],vec![0x35,0x36]];
        let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve("PlayersHouse2F",&refs,&g,None);
        let p=placements.iter().find(|p|p.kind==ModelKind::Bookcase).unwrap();
        let mut mesh=TerrainMeshData{authored_cells:vec![None;tiles.len()],
            footing_heights:vec![2.5;tiles.len()],..Default::default()};
        assert!(append(&mut mesh,"PlayersHouse2F",&refs,&g,p,&mut vec![false;tiles.len()],[0,0]));
        assert!(mesh.solid.positions.is_empty(),"no pastel book paint");
        assert!(mesh.textured.colors.iter().all(|c|c[0]==c[1] && c[1]==c[2]));
        assert_eq!(mesh.footing_heights,vec![2.5;tiles.len()]);
        tiles[0].source.tile_index=0xffff;
        assert!(resolve("PlayersHouse2F",&tiles.iter().collect::<Vec<_>>(),&g,None)
            .iter().all(|p|p.kind!=ModelKind::Bookcase));
    }

    #[test]
    fn native_house_bookcase_uses_own_cover_and_page_source_without_pastel_paint() {
        let mut object=profile("Shared facility paired bookcases 06");
        object.map=None;object.maps=None;object.parts.clear();object.metatiles=None;
        object.tileset="house".into();object.metatile=4;object.origin=[0,0];object.ground=1;
        object.tiles=vec![vec![0x32,0x33,0x32,0x33],vec![0x30,0x31,0x0e,0x0f],
            vec![0x0e,0x0f,0x0e,0x0f],vec![0x3c,0x3b,0x1e,0x1f]];
        let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve("BluesHouse",&refs,&g,None);
        let books=placements.iter().filter(|p|p.kind==ModelKind::Bookcase).collect::<Vec<_>>();
        assert!(!books.is_empty());
        let mut mesh=TerrainMeshData{authored_cells:vec![None;tiles.len()],footing_heights:vec![2.5;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        for p in books {
            assert!(append(&mut mesh,"BluesHouse",&refs,&g,p,&mut claimed,[0,0]));
            assert!(p.indices(g.width).all(|i|mesh.authored_cells[i]==Some("interior/bookcase")));
        }
        assert!(mesh.solid.positions.is_empty(),"books must use the house palette instead of pastel paint");
        assert!(!mesh.textured.positions.is_empty());
        assert!(mesh.textured.colors.iter().all(|c|c[0]==c[1]&&c[1]==c[2]));
        assert_eq!(mesh.footing_heights,vec![2.5;tiles.len()]);
    }
    #[test]
    fn kanto_native_bedroom_pc_keeps_complete_source_claim_and_own_palette() {
        let mut object=profile("Player bedroom computer feet");
        object.map=None;object.maps=None;object.parts.clear();object.metatiles=None;
        object.tileset="players_house".into();object.metatile=0x10;object.origin=[0,1];object.ground=1;
        object.tiles=vec![vec![0x20,0x21],vec![0x30,0x31],vec![0x0c,0x0d]];
        let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve("RedsHouse2F",&refs,&g,None);
        let p=placements.iter().find(|p|p.kind==ModelKind::Computer).unwrap();
        let mut mesh=TerrainMeshData{authored_cells:vec![None;tiles.len()],footing_heights:vec![2.5;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        assert!(append(&mut mesh,"RedsHouse2F",&refs,&g,p,&mut claimed,[0,0]));
        assert!(mesh.solid.positions.is_empty());
        assert_eq!(mesh.footing_heights,vec![2.5;tiles.len()]);
        assert!(p.indices(g.width).all(|i|claimed[i] && mesh.authored_cells[i]==Some("interior/computer")));
        for uv in mesh.textured.uvs.iter().skip(4*p.width*p.height) {
            assert!(uv[0]<2./g.width as f32 && uv[1]<3./g.height as f32);
        }
        let mut clipped=tiles.clone();clipped[0].source.tile_index=0x0a;
        assert!(!resolve("RedsHouse2F",&clipped.iter().collect::<Vec<_>>(),&g,None)
            .iter().any(|p|p.kind==ModelKind::Computer));
    }
    #[test]
    fn missing_selected_interior_keeps_source_atlas_and_prevents_legacy_extrusion() {
        let g=GridGeometry{width:3,height:2,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let p=Placement{kind:ModelKind::Chair,column:1,row:0,width:2,height:2,
            ground:0,depth_pixels:16.,height_pixels:12.,base_pixels:0.,footing_pixels:None};
        let mut mesh=TerrainMeshData{footing_heights:vec![3.;6],authored_cells:vec![None;6],..Default::default()};
        let mut claimed=vec![false;6];
        append_missing_source(&mut mesh,&g,&p,&mut claimed);
        assert_eq!(claimed,vec![false,true,true,false,true,true]);
        assert!(mesh.solid.positions.is_empty());
        assert_eq!(mesh.footing_heights,vec![3.;6]);
        assert_eq!(mesh.authored_cells,vec![None;6]);
        assert_eq!(mesh.textured.positions.len(),16);
        for (face,i) in p.indices(g.width).enumerate() {
            let (u0,u1,v0,v1)=g.uv(i%g.width,i/g.width);
            assert_eq!(&mesh.textured.uvs[face*4..face*4+4],&[[u0,v0],[u0,v1],[u1,v1],[u1,v0]]);
        }
    }
    #[test]
    fn arcade_bank_screens_face_the_west_and_east_seats() {
        let mut mesh = SurfaceMeshData::default();
        // Deliberately rectangular footprint catches rotations after fitting.
        append_outward_arcade_pair(&mut mesh, [10.0, 42.0, 20.0, 34.0], 0.0, 21.0);
        assert!(mesh.positions.iter().all(|p|
            p[0] >= 9.999 && p[0] <= 42.001 && p[2] >= 19.999 && p[2] <= 34.001));
        let mut screens = [0, 0];
        for ((p, n), color) in mesh.positions.iter().zip(&mesh.normals).zip(&mesh.colors) {
            assert!((Vec3::from_array(*n).length() - 1.0).abs() < 0.001);
            if (color[0] / color[1] - 0.11 / 0.27).abs() < 0.001
                && (color[1] / color[2] - 0.27 / 0.29).abs() < 0.001 {
                let side = usize::from(p[0] > 26.0);
                assert!(if side == 0 { p[0] < 18.0 } else { p[0] > 34.0 }, "screen faces the bank interior");
                if n[0] * (if side == 0 { -1.0 } else { 1.0 }) > 0.99 { screens[side] += 1; }
            }
        }
        assert!(screens.iter().all(|&count| count > 0));
    }
    fn profile(name: &str) -> Object {
        built_in_profiles()
            .objects
            .iter()
            .find(|o| o.name == name)
            .unwrap()
            .clone()
    }
    #[test]
    fn kanto_native_mansion_lounge_table_owns_complete_gold_drawing_and_keeps_footing() {
        let mut object=profile("Shared lab display table");
        object.tileset="mansion".into();object.metatile=0x10;object.metatiles=None;
        object.origin=[0,0];object.ground=0x11;object.parts.clear();object.footing_pixels=None;
        object.tiles=vec![vec![0x36,0x37,0x37,0x38],vec![0x46,0x47,0x47,0x48],
            vec![0x46,0x47,0x47,0x48],vec![0x56,0x57,0x57,0x58]];
        let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        let p=resolve("CeladonMansion1F",&refs,&g,None).into_iter()
            .find(|p|native_mansion_lounge_table("CeladonMansion1F",p,&refs,&g)).unwrap();
        let mut mesh=TerrainMeshData{footing_heights:vec![3.25;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        assert!(append(&mut mesh,"CeladonMansion1F",&refs,&g,&p,&mut claimed,[0,0]));
        assert_eq!(claimed.iter().filter(|&&v|v).count(),16);
        assert_eq!(mesh.footing_heights,vec![3.25;tiles.len()]);
        assert!(mesh.solid.positions.is_empty());
        assert!(mesh.textured.positions.iter().all(|v|v[1]<=10.001));
        let swatch=native_source_swatches(&p,&g,[[12,12]])[0];
        assert_native_tabletop_projection(&mesh.textured,&g,&p);
        assert!(mesh.textured.indices.iter().map(|&i|i as usize)
            .filter(|&i|i>=16*4 && mesh.textured.normals[i][1]<0.99)
            .all(|i|mesh.textured.uvs[i]==swatch));
        for i in p.indices(g.width) {
            let mut changed=tiles.clone();changed[i].source.tile_index^=0x80;
            assert!(!resolve("CeladonMansion1F",&changed.iter().collect::<Vec<_>>(),&g,None)
                .iter().any(|p|p.kind==ModelKind::DiningTable));
        }
        for map in ["CeladonMansion2F","GoldenrodHouse"] {
            assert!(!resolve(map,&refs,&g,None).iter().any(|p|p.kind==ModelKind::DiningTable));
        }
        object.name="Custom lounge table".into();
        assert!(!resolve("CeladonMansion1F",&refs,&g,Some(&Document{objects:vec![object],atmosphere:None}))
            .iter().any(|p|p.kind==ModelKind::DiningTable));
    }
    #[test]
    fn kanto_native_oak_desk_has_no_invented_specimens_and_preserves_source_ownership() {
        let object=profile("Shared lab display table");
        let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        let p=resolve("OaksLab",&refs,&g,None).into_iter()
            .find(|p|native_oak_table("OaksLab",p,&refs,&g)).unwrap();
        assert_eq!(p.height_pixels,10.);
        assert_eq!(p.footing_pixels,object.footing_pixels);
        let elm=resolve("ElmsLab",&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::DiningTable).unwrap();
        assert_eq!(traditional_tabletop_height("ElmsLab",&refs,&g,None,elm.column,elm.row),Some(10.));
        let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        assert!(append(&mut mesh,"OaksLab",&refs,&g,&p,&mut claimed,[0,0]));
        assert_eq!(claimed.iter().filter(|&&v|v).count(),18);
        let ordinary=resolve("UnreviewedLab",&refs,&g,None).into_iter()
            .find(|p|p.kind==ModelKind::LabDisplayTable).unwrap();
        let mut old=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
        assert!(append(&mut old,"UnreviewedLab",&refs,&g,&ordinary,&mut vec![false;tiles.len()],[0,0]));
        assert_eq!(mesh.footing_heights,old.footing_heights);
        assert!(mesh.solid.positions.is_empty());
        assert!(mesh.textured.positions.iter().all(|v|v[1]<=10.001));
        let swatch=native_source_swatches(&p,&g,[[10,4]])[0];
        assert_native_tabletop_projection(&mesh.textured,&g,&p);
        assert!(mesh.textured.indices.iter().map(|&i|i as usize)
            .filter(|&i|i>=18*4 && mesh.textured.normals[i][1]<0.99)
            .all(|i|mesh.textured.uvs[i]==swatch));
        for i in p.indices(g.width) {
            let mut changed=tiles.clone();changed[i].source.tile_index^=0x80;
            assert!(!resolve("OaksLab",&changed.iter().collect::<Vec<_>>(),&g,None)
                .iter().any(|p|p.kind==ModelKind::DiningTable && p.width==6));
        }
        let mut custom=object.clone();custom.name="Custom desk".into();
        assert!(!resolve("OaksLab",&refs,&g,Some(&Document{objects:vec![custom],atmosphere:None}))
            .iter().any(|p|p.kind==ModelKind::DiningTable && p.width==6));
    }
    #[test]
    fn elm_external_pack_has_native_table_workstation_and_both_gray_fixtures() {
        let explicit=std::env::var_os("CRYSTAL_RENDER_TEST_PACK");
        let path=explicit.as_ref().map(std::path::PathBuf::from).unwrap_or_else(||
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        assert!(explicit.is_none()||path.is_file());if !path.is_file(){return;}
        let pack=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let module=&pack.pack().data().maps["ElmsLab"];
        let layout=&pack.pack().runtime_files()["data/tilesets/lab_metatiles.bin"];
        let (w,h)=(module.attributes.width as usize*4,module.attributes.height as usize*4);
        let mut tiles=Vec::new();
        for y in 0..h {for x in 0..w {
            let block=module.blocks[y/4*(w/4)+x/4];
            tiles.push(VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                source:VisualTileSource{tileset_id:"lab".into(),metatile_id:block,subtile_column:(x%4)as u8,
                    subtile_row:(y%4)as u8,tile_index:layout[block as usize*16+y%4*4+x%4]as u16}});
        }}
        let refs=tiles.iter().collect::<Vec<_>>();
        let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let placements=resolve("ElmsLab",&refs,&g,None);
        for kind in [ModelKind::DiningTable,ModelKind::LabWorkstation,ModelKind::LabRestorationMachine,ModelKind::LabCounter] {
            let found=placements.iter().filter(|p|p.kind==kind).collect::<Vec<_>>();
            assert_eq!(found.len(),1,"{kind:?}");
            for p in found {
            match kind {
                ModelKind::DiningTable=>{assert!(native_oak_table("ElmsLab",p,&refs,&g));
                    for column in [12,14,16] {assert_eq!(traditional_tabletop_height("ElmsLab",&refs,&g,None,column,6),Some(10.));}},
                ModelKind::LabWorkstation=>assert!(native_lab_workstation_source("ElmsLab",p,&refs,&g).is_some()),
                ModelKind::LabCounter=>assert!(native_elm_central_source("ElmsLab",p,&refs,&g).is_some()),
                _=>assert!(native_elm_fixture_source("ElmsLab",p,&refs,&g).is_some()),
            }
            let mut mesh=TerrainMeshData{footing_heights:vec![2.75;tiles.len()],..Default::default()};
            let mut claims=vec![false;tiles.len()];assert!(append(&mut mesh,"ElmsLab",&refs,&g,p,&mut claims,[0,0]));
            assert!(mesh.solid.indices.is_empty(),"{kind:?}: imported substitute colour remains");
            assert_eq!(claims.iter().filter(|&&x|x).count(),p.width*p.height);
            let mut support=vec![2.75;tiles.len()];
            if let Some(height)=p.footing_pixels {for i in p.indices(w){support[i]=height;}}
            assert_eq!(mesh.footing_heights,support);
            if kind==ModelKind::LabCounter {
                assert!(mesh.textured.positions.iter().all(|v|v[1]<=11.501),"low phone/console, not a full monitor and invented chair");
                let bitmap=image::load_from_memory(&pack.pack().runtime_files()["gfx/tilesets/lab.png"]).unwrap().to_luma8();
                let count=(bitmap.width()/8)*(bitmap.height()/8);
                for (pixel,value) in [([18,8],0),([28,8],85),([0,8],170),([19,8],255)] {
                    let source=&refs[(p.row+pixel[1]/8)*w+p.column+pixel[0]/8].source;
                    let packed=pack.pack().data().tilesets["lab"].palette_map[source.tile_index as usize];
                    let tile=(source.tile_index as u32&127)+if packed&8!=0{count/2}else{0};
                    assert_eq!(bitmap.get_pixel(tile%16*8+(pixel[0]%8)as u32,tile/16*8+(pixel[1]%8)as u32).0[0],value,
                        "central desk swatches must resolve the source VRAM bank");
                }
            }
            assert!(mesh.textured.positions.iter().all(|v|v[0]>=p.column as f32*8.-0.001
                &&v[0]<=(p.column+p.width)as f32*8.+0.001&&v[2]>=p.row as f32*8.-0.001
                &&v[2]<=(p.row+p.height)as f32*8.+0.001),"{kind:?}: geometry must stay inside its source footprint");
            }
        }
        assert!(!placements.iter().any(|p|p.kind==ModelKind::LabDisplayTable),"fixed specimens must not duplicate live starter balls");
    }
    #[test]
    fn elm_equipment_uses_complete_sources_and_preserves_support_and_custom_ownership() {
        for name in ["Lab restoration machine","Lab central desk assembly"] {
            let object=profile(name);let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let p=resolve("ElmsLab",&refs,&g,None).into_iter().find(|p|
                matches!(p.kind,ModelKind::LabRestorationMachine|ModelKind::LabCounter)).unwrap();
            let swatches=if p.kind==ModelKind::LabCounter {
                native_elm_central_source("ElmsLab",&p,&refs,&g).unwrap().to_vec()
            }else{native_elm_fixture_source("ElmsLab",&p,&refs,&g).unwrap().to_vec()};
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
            let mut claims=vec![false;tiles.len()];assert!(append(&mut mesh,"ElmsLab",&refs,&g,&p,&mut claims,[0,0]));
            assert!(mesh.solid.indices.is_empty());assert_eq!(claims.iter().filter(|&&x|x).count(),16);
            assert_eq!(mesh.footing_heights,vec![1.25;tiles.len()]);
            if p.kind==ModelKind::LabRestorationMachine {
                assert!(mesh.textured.uvs.contains(&swatches[0]),"dark recessed bed");
                assert!(mesh.textured.positions.iter().all(|v|v[1]<=8.001));
            }else{
                assert!(mesh.textured.positions.iter().all(|v|v[1]<=11.501));
                assert!(mesh.textured.positions.iter().any(|v|v[1]>10.));
            }
            assert!(native_elm_fixture_source("OaksLab",&p,&refs,&g).is_none());
            let mut custom=object.clone();custom.name="User lab equipment".into();
            assert!(!resolve("ElmsLab",&refs,&g,Some(&Document{objects:vec![custom],atmosphere:None}))
                .iter().any(|q|q.kind==p.kind));
            drop(refs);
            for i in p.indices(g.width) {
                let old=tiles[i].source.tile_index;tiles[i].source.tile_index=0xffff;
                let changed=tiles.iter().collect::<Vec<_>>();
                assert!(native_elm_fixture_source("ElmsLab",&p,&changed,&g).is_none());
                assert!(native_elm_central_source("ElmsLab",&p,&changed,&g).is_none());
                tiles[i].source.tile_index=old;
            }
        }
    }
    #[test]
    fn elm_healing_machine_has_open_recess_low_front_and_correct_face_winding() {
        let object=profile("Lab restoration machine");let (tiles,mut g)=fixture(&object);
        let refs=tiles.iter().collect::<Vec<_>>();
        let p=resolve("ElmsLab",&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::LabRestorationMachine).unwrap();
        assert_eq!((p.depth_pixels,p.height_pixels),(24.,8.));
        for scale in [0.5,1.,2.] {
            g.tile_width=8.*scale;g.tile_height=8.*scale;
            let swatches=native_elm_fixture_source("ElmsLab",&p,&refs,&g).unwrap();
            let mut out=SurfaceMeshData::default();
            append_native_elm_healing_machine(&mut out,&g,&p,[10.,10.+32.*scale,20.,20.+32.*scale],swatches);
            assert_eq!(out.positions.iter().map(|v|v[1]).fold(0.,f32::max),8.*scale);
            let depth=out.positions.iter().map(|v|v[2]).fold(f32::NEG_INFINITY,f32::max)
                -out.positions.iter().map(|v|v[2]).fold(f32::INFINITY,f32::min);
            assert!((depth-24.*scale).abs()<0.001);
            let sample=Vec2::new(10.+16.1*scale,20.+22.3*scale);let mut hits=Vec::new();
            for tri in out.indices.chunks_exact(3) {
                let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(out.positions[i as usize]));
                let normal=Vec3::from_array(out.normals[tri[0]as usize]);
                assert!((b-a).cross(c-a).dot(normal)>0.,"outward nondegenerate faces");
                if normal.y>0.99 {
                    let [a2,b2,c2]=[a,b,c].map(|p|Vec2::new(p.x,p.z));
                    let area=(b2-a2).perp_dot(c2-a2);
                    let u=(b2-sample).perp_dot(c2-sample)/area;
                    let v=(c2-sample).perp_dot(a2-sample)/area;
                    if u>0.&&v>0.&&u+v<1. {hits.push(a.y);}
                }
            }
            assert_eq!(hits.len(),1,"recess must have one floor and no lid covering it");
            assert!((hits[0]-5.5*scale).abs()<0.001);
            let front=g.uv(p.column,p.row+3);
            assert!(out.uvs.iter().zip(&out.normals).zip(&out.positions).any(|((uv,n),v)|
                *uv==[front.0,front.2]&&n[2]>0.99&&(v[1]-7.4*scale).abs()<0.001),"last source row belongs on front");
            let roof=g.uv(p.column,p.row);
            assert!(out.uvs.iter().zip(&out.normals).zip(&out.positions).any(|((uv,n),v)|
                *uv==[roof.0,roof.2]&&n[1]>0.99&&(v[1]-8.*scale).abs()<0.001),"source roof belongs on top");
        }
        let mut legacy=TerrainMeshData::default();
        // The canonical source profile is Elm-only. Exercise the generic model
        // fallback directly, without inventing a machine placement in Oak's lab.
        assert!(!resolve("OaksLab",&refs,&g,None).iter().any(|p|p.kind==ModelKind::LabRestorationMachine));
        let legacy_p=Placement{depth_pixels:12.,height_pixels:22.,..p.clone()};
        assert!(append(&mut legacy,"UnreviewedLab",&refs,&g,&legacy_p,&mut vec![false;tiles.len()],[0,0]));
        assert!(!legacy.solid.indices.is_empty(),"other room callers retain their authored restoration model");
    }
    #[test]
    fn kanto_native_lab_cabinet_routes_structure_to_atlas_and_respects_source_ownership() {
        let mut object=profile("Shared lab paired bookcases 14");
        let (mut tiles,g)=fixture(&object);
        let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve("OaksLab",&refs,&g,None);
        let cabinet=placements.iter().find(|p|p.kind==ModelKind::LabBookcase).unwrap();
        let build=|map| {
            let mut mesh=TerrainMeshData{footing_heights:vec![2.5;tiles.len()],..Default::default()};
            assert!(append(&mut mesh,map,&refs,&g,cabinet,&mut vec![false;tiles.len()],[0,0]));
            mesh
        };
        let native=build("OaksLab");let ordinary=build("UnreviewedLab");
        assert!(native.textured.indices.len()>ordinary.textured.indices.len());
        assert!(native.solid.indices.len()<ordinary.solid.indices.len());
        assert_eq!(native.textured.indices.len()+native.solid.indices.len(),
            ordinary.textured.indices.len()+ordinary.solid.indices.len());
        assert_eq!(native.footing_heights,ordinary.footing_heights);
        assert!(native.solid.indices.is_empty(),"native lab books must use their source palette");
        assert!(native.textured.colors.iter().all(|c|c[0]==c[1]&&c[1]==c[2]));
        for map in ["ElmsLab","EarlsPokemonAcademy","DragonShrine"] {
            let johto=build(map);
            assert_eq!(johto.textured.uvs,native.textured.uvs);
            assert_eq!(johto.textured.positions,native.textured.positions);
            assert!(johto.solid.indices.is_empty());
            let mut custom=object.clone();custom.name="User lab shelf".into();
            assert!(!resolve(map,&refs,&g,Some(&Document{objects:vec![custom],atmosphere:None}))
                .iter().any(|p|p.kind==ModelKind::LabBookcase));
        }
        object.name="User cabinet".into();
        assert!(!resolve("OaksLab",&refs,&g,Some(&Document{objects:vec![object],atmosphere:None}))
            .iter().any(|p|p.kind==ModelKind::LabBookcase));
        tiles[0].source.tile_index=0xffff;
        assert!(!resolve("OaksLab",&tiles.iter().collect::<Vec<_>>(),&g,None)
            .iter().any(|p|p.kind==ModelKind::LabBookcase && p.column==0));
    }
    #[test]
    fn arcade_stool_seat_is_smaller_than_its_native_plot() {
        let object=profile("Shared game corner stool 5 0");
        let (tiles,g)=fixture(&object);
        let cells:Vec<_>=tiles.iter().collect();
        let p=resolve("GoldenrodGameCorner",&cells,&g,None);
        assert_eq!(p.len(),1);
        let mut mesh=TerrainMeshData {footing_heights:vec![0.0;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        assert!(append(&mut mesh,"GoldenrodGameCorner",&cells,&g,&p[0],&mut claimed,[0,0]));
        let min=mesh.solid.positions.iter().map(|p|p[0]).fold(f32::INFINITY,f32::min);
        let max=mesh.solid.positions.iter().map(|p|p[0]).fold(f32::NEG_INFINITY,f32::max);
        assert!((max-min-8.0).abs()<0.001);
        assert_eq!(claimed.iter().filter(|&&x|x).count(),4);
        assert_eq!(mesh.footing_heights,vec![0.0;tiles.len()]);
    }
    #[test]
    fn new_user_casino_profiles_keep_priority_over_bundled_cabinets() {
        let mut object=profile("Shared game row cabinet 7 0");
        let (tiles,g)=fixture(&object);
        object.name="User bespoke cabinet".into();
        let active=Document {objects:vec![object],atmosphere:None};
        assert!(resolve("GoldenrodGameCorner",&tiles.iter().collect::<Vec<_>>(),&g,Some(&active)).is_empty());
    }
    #[test]
    fn department_store_plant_is_not_a_merchandise_fallback() {
        let mut object=profile("Shared department store cushion 2");
        object.name="Reviewed department store west plant".into();
        object.metatile=0x0b;
        object.metatiles=None;
        object.origin=[0,0];
        object.tiles=vec![vec![0x4a,0x4b],vec![0x08,0x09],vec![0x89,0x8a],vec![0xa7,0xa8]];
        object.ground=0x01;
        let (mut tiles,g)=fixture(&object);
        for map in ["GoldenrodDeptStore1F","CeladonDeptStore1F","GoldenrodDeptStore6F","CeladonDeptStore6F"] {
            let refs=tiles.iter().collect::<Vec<_>>();
            let plants=resolve(map,&refs,&g,None);
            assert_eq!(plants.len(),1);
            assert_eq!(plants[0].kind,ModelKind::PottedPlant);
            let mut mesh=TerrainMeshData {footing_heights:vec![0.;tiles.len()],..Default::default()};
            assert!(append(&mut mesh,map,&refs,&g,&plants[0],&mut vec![false;tiles.len()],[0,0]));
            assert!(mesh.solid.positions.is_empty());
            assert!(mesh.textured.positions.iter().any(|p| p[1]>20.));
            assert!(native_store_plant_source(map,&plants[0],&refs,&g));
            assert!(!native_store_plant_source("UnreviewedStore",&plants[0],&refs,&g));
            let mut raised=plants[0].clone();raised.base_pixels=4.;
            assert!(!native_store_plant_source(map,&raised,&refs,&g));
            assert_eq!(mesh.footing_heights,vec![0.;tiles.len()]);
            for index in [0,1,g.width,g.width+1,2*g.width,2*g.width+1,3*g.width,3*g.width+1] {
                let old=tiles[index].source.tile_index;
                tiles[index].source.tile_index=0xffff;
                assert!(resolve(map,&tiles.iter().collect::<Vec<_>>(),&g,None).is_empty());
                tiles[index].source.tile_index=old;
            }
        }
        let mut unknown=tiles[0].source.clone();
        unknown.metatile_id=0xffff;
        assert_eq!(mart_display_model(&unknown),None);
    }
    #[test]
    fn fourth_floor_shelf_reveal_keeps_independent_cabinets_and_floor_support() {
        let mut object=profile("Shared department store cushion 2");
        object.metatile=0x13;object.metatiles=None;object.origin=[0,0];object.ground=1;
        object.tiles=vec![vec![0x40,0x41,0x42,0x2b],vec![0x50,0x51,0x52,0x45],
            vec![0x43,0x44,0x5c,0x5d],vec![0x53,0x54,0x1f,0x55]];
        for block in [0x13,0x15] {
        if block==0x15 {
            object.metatile=block;
            object.tiles=vec![vec![12,13,12,13],vec![80,81,80,81],
                vec![80,81,80,81],vec![94,95,94,95]];
        }
        let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        for map in ["GoldenrodDeptStore4F","CeladonDeptStore4F"] {
            let cabinets=resolve(map,&refs,&g,None);
            assert_eq!(cabinets.len(),2);
            let mut mesh=TerrainMeshData{footing_heights:vec![2.25;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            for p in &cabinets {
                assert!(native_store_aisle_shelf_source(map,p,&refs,&g));
                let joins=native_store_shelf_neighbors(map,p,&refs,&g);
                assert_eq!(joins,if p.column==0 {[false,true]}else{[true,false]});
                assert!(append(&mut mesh,map,&refs,&g,p,&mut claimed,[0,0]));
                assert!(!native_store_aisle_shelf_source("GoldenrodDeptStore3F",p,&refs,&g));
                let mut raised=p.clone();raised.base_pixels=4.;
                assert!(!native_store_aisle_shelf_source(map,&raised,&refs,&g));
                for i in p.indices(g.width) {
                    let mut changed=tiles.clone();changed[i].source.tile_index^=1;
                    assert!(!native_store_aisle_shelf_source(map,p,&changed.iter().collect::<Vec<_>>(),&g));
                }
            }
            let ranges=&mesh.textured.cutaway_ranges;
            assert_eq!(ranges.len(),2);
            assert_eq!(&mesh.depth_coverage_textured_ranges,ranges);
            assert!(ranges[0].end<ranges[1].start,"opaque floor separates the cabinets");
            assert_eq!(claimed.iter().filter(|&&v|v).count(),16);
            assert!(claimed.iter().enumerate().all(|(i,&v)|v==(i%g.width<4&&i/g.width<4)));
            assert_eq!(mesh.footing_heights,vec![2.25;tiles.len()]);
            let seam=g.bounds(2,0).0;
            assert!(!mesh.textured.indices.chunks_exact(3).any(|tri|tri.iter().all(|&i|
                mesh.textured.positions[i as usize][0]==seam
                    &&mesh.textured.normals[i as usize][0].abs()==1.)),"no internal gray side walls");
            for (x,normal) in [(g.bounds(0,0).0,-1.),(g.bounds(3,0).1,1.)] {
                assert!(mesh.textured.indices.chunks_exact(3).any(|tri|tri.iter().all(|&i|
                    mesh.textured.positions[i as usize][0]==x
                        &&mesh.textured.normals[i as usize][0]==normal)),"outer bank side stays closed");
            }
        }
        }
    }
    #[test]
    fn store_electronics_preserve_both_native_fronts_on_closed_cabinets() {
        let mut object=profile("Shared department store cushion 2");
        object.metatile=0x13;object.metatiles=None;object.origin=[0,0];object.ground=1;
        object.tiles=vec![vec![0x40,0x41,0x42,0x2b],vec![0x50,0x51,0x52,0x45],
            vec![0x43,0x44,0x5c,0x5d],vec![0x53,0x54,0x1f,0x55]];
        let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        for map in ["GoldenrodDeptStore3F","CeladonDeptStore3F"] {
            let displays=resolve(map,&refs,&g,None);
            let cabinets=displays.iter().filter(|p|p.kind==ModelKind::RetailShelf).collect::<Vec<_>>();
            assert_eq!(cabinets.len(),2);
            for p in cabinets {
                let mut mesh=SurfaceMeshData::default();
                append_native_retail_display(&mut mesh,&g,p,[0.,16.,0.,10.],false);
                let mut volume=0.;let mut front_area=0.;
                for tri in mesh.indices.chunks_exact(3) {
                    let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
                    let cross=(points[1]-points[0]).cross(points[2]-points[0]);
                    assert!(cross.dot(Vec3::from_array(mesh.normals[tri[0]as usize]))>0.);
                    volume+=points[0].dot(points[1].cross(points[2]))/6.;
                    if mesh.normals[tri[0]as usize][2]>0.5 {front_area+=cross.z/2.;}
                }
                assert!(volume>16.*10.*22.5*0.90 && volume<16.*10.*22.5,
                    "closed cabinet volume excludes the perspective roof row");
                assert!((front_area-16.*22.5).abs()<0.01);
                assert!(mesh.positions.iter().any(|p|p[2]>9. && p[2]<10.));
                for y in 0..4 {for x in 0..2 {
                    let uv=g.uv(p.column+x,p.row+y);
                    assert!(mesh.uvs.contains(&[uv.0,uv.2]));
                    assert!(mesh.uvs.contains(&[uv.1,uv.3]));
                }}
            }
        }
    }
    #[test]
    fn sixth_floor_vending_uses_complete_gray_source_and_preserves_footing() {
        for origin in [0,2] {
            let mut object=profile("Shared department store cushion 2");
            object.name="Vending source fixture".into();object.metatile=0x21;object.metatiles=None;
            object.origin=[origin,0];object.ground=1;
            object.tiles=vec![vec![0x0c,0x0d],vec![0x4c,0x4d],vec![0x4c,0x4d],vec![0x4e,0x4f]];
            let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            for map in ["GoldenrodDeptStore6F","CeladonDeptStore6F"] {
                let placements=resolve(map,&refs,&g,None);
                let p=placements.iter().find(|p|p.kind==ModelKind::VendingMachine).unwrap();
                assert!(native_store_vending_source(map,p,&refs,&g));
                let mut mesh=TerrainMeshData{footing_heights:vec![2.75;tiles.len()],..Default::default()};
                let mut claimed=vec![false;tiles.len()];
                assert!(append(&mut mesh,map,&refs,&g,p,&mut claimed,[0,0]));
                assert!(mesh.solid.positions.is_empty());assert!(!mesh.textured.positions.is_empty());
                assert_eq!(claimed.iter().filter(|&&v|v).count(),8);
                assert_eq!(mesh.footing_heights,vec![2.75;tiles.len()]);
                let top=p.height_pixels*g.tile_height/8.;
                let vent_floor=top-0.22*g.tile_height/8.;
                let dark=native_source_swatches(p,&g,[[0,8]])[0];
                let wells=mesh.textured.indices.chunks_exact(3).filter(|tri|tri.iter().all(|&i| {
                    let i=i as usize;
                    (mesh.textured.positions[i][1]-vent_floor).abs()<0.001
                        &&mesh.textured.normals[i][1]>0.999&&mesh.textured.uvs[i]==dark
                })).count();
                assert_eq!(wells,12,"six genuine recessed vent floors");
                assert!(mesh.textured.indices.iter().all(|&i|mesh.textured.positions[i as usize][1]<=top+0.001));
                for i in p.indices(g.width) {
                    let mut changed=tiles.clone();changed[i].source.tile_index^=1;
                    assert!(!native_store_vending_source(map,p,&changed.iter().collect::<Vec<_>>(),&g));
                    changed=tiles.clone();changed[i].source.subtile_column^=2;
                    assert!(!native_store_vending_source(map,p,&changed.iter().collect::<Vec<_>>(),&g));
                }
                assert!(!native_store_vending_source("UnreviewedStore6F",p,&refs,&g));
                assert!(!native_station_ticket_source("GoldenrodMagnetTrainStation",p,&refs,&g));
                let mut raised=p.clone();raised.base_pixels=4.;
                assert!(!native_store_vending_source(map,&raised,&refs,&g));
            }
        }
    }
    #[test]
    fn sixth_floor_right_seats_match_the_left_seating_height() {
        let mut object=profile("Shared department store cushion 2");
        object.metatile=0x20;
        object.metatiles=None;
        object.origin=[2,0];
        object.tiles=vec![vec![0x22,0x23],vec![0x32,0x33],vec![0x24,0x25],vec![0x34,0x35]];
        object.ground=0x01;
        let (mut tiles,g)=fixture(&object);
        for map in ["GoldenrodDeptStore6F","CeladonDeptStore6F"] {
            let seats=resolve(map,&tiles.iter().collect::<Vec<_>>(),&g,None);
            assert_eq!(seats.len(),1);
            assert_eq!(seats[0].kind,ModelKind::Cushion);
            assert_eq!((seats[0].depth_pixels,seats[0].height_pixels),(12.,7.));
            let old=tiles[0].source.tile_index;
            tiles[0].source.tile_index=0xffff;
            assert!(resolve(map,&tiles.iter().collect::<Vec<_>>(),&g,None).is_empty());
            tiles[0].source.tile_index=old;
        }
    }
    #[test]
    fn department_store_seating_uses_native_height_and_keeps_support() {
        for object in built_in_profiles().objects.iter().filter(|o| o.tileset=="mart" && o.name.starts_with("Shared department store cushion")) {
            let (mut tiles,g)=fixture(object);
            let cells=tiles.iter().collect::<Vec<_>>();
            let p=resolve("GoldenrodDeptStore1F",&cells,&g,None);
            let seat=p.iter().find(|p|p.kind==ModelKind::Cushion).unwrap();
            assert_eq!(seat.height_pixels,7.0);
            assert_eq!(seat.depth_pixels,12.0);
            let mut mesh=TerrainMeshData {footing_heights:vec![2.5;tiles.len()],..Default::default()};
            assert!(append(&mut mesh,"GoldenrodDeptStore1F",&cells,&g,seat,&mut vec![false;tiles.len()],[0,0]));
            assert!(mesh.footing_heights.iter().all(|h|*h==2.5));
            assert!(mesh.solid.positions.is_empty());
            let top=mesh.textured.positions.iter().map(|p|p[1]).fold(0.0,f32::max);
            assert!((top-7.0).abs()<0.001);
            assert!(mesh.textured.positions.iter().any(|p|p[1]==0.0));
            let swatches=native_source_swatches(seat,&g,[[1,1],[3,3],[0,1]]);
            assert!(mesh.textured.positions.iter().zip(&mesh.textured.uvs)
                .filter(|(position,_)|position[1]>0.)
                .all(|(_,uv)|swatches.contains(uv)));
            assert!(native_store_seat_source("GoldenrodDeptStore1F",seat,&cells,&g));
            assert!(!native_store_seat_source("UnreviewedStore",seat,&cells,&g));
            let mut raised=seat.clone();raised.base_pixels=4.;
            assert!(!native_store_seat_source("GoldenrodDeptStore1F",&raised,&cells,&g));
            tiles[0].source.tile_index=0xffff;
            assert!(resolve("GoldenrodDeptStore1F",&tiles.iter().collect::<Vec<_>>(),&g,None).iter().all(|p|p.kind!=ModelKind::Cushion));
        }
    }
    #[test]
    fn automatic_seating_uses_complete_chair_and_stool_claims() {
        use bevy::prelude::{UVec2,Vec2};
        for (name, map, expected_height) in [("Shared station seat 0", "GoldenrodMagnetTrainStation", None),
            ("Shared game corner stool 5 0", "GoldenrodGameCorner", None),
            ("Shared lab round stool", "ElmsLab", None),
            ("Shared game corner stool 5 0", "CeladonCafe", Some(5.)),
            ("Shared Pokecenter seat 2e", "PewterPokecenter1F", Some(7.))] {
            let object = profile(name);
            let (tiles, g) = fixture(&object);
            let mut frame = VisualWorldFrame::default();
            frame.map_id = map.into();
            frame.grid_size = UVec2::new(g.width as u32, g.height as u32);
            frame.tile_size = Vec2::splat(8.);
            frame.viewport_size = frame.tile_size * frame.grid_size.as_vec2();
            frame.center = Vec2::new(frame.viewport_size.x * 0.5, -frame.viewport_size.y * 0.5);
            frame.tiles = tiles;
            let seats = super::super::seating_surfaces(&frame, None);
            assert_eq!(seats.len(), 1, "{name}");
            if let Some(height) = expected_height {
                assert!((seats[0].height - height).abs() < 0.001, "{map}: native cushion height");
            }
            let foot = Vec2::new(8., -16.);
            assert!(seats[0].target(&frame, foot, 0.).is_some(), "{name}");
            assert!(seats[0].target(&frame, foot + Vec2::new(16., 0.), 0.).is_none());
            // One changed source cell invalidates the whole furniture drawing.
            frame.tiles[0].source.tile_index ^= 0x80;
            assert!(super::super::seating_surfaces(&frame, None).is_empty(), "{name}");
        }
    }

    fn fixture(object: &Object) -> (Vec<VisualTile>, GridGeometry) {
        let width = object.tiles[0].len() + 2;
        let height = object.tiles.len() + 2;
        let geometry = GridGeometry {
            width,
            height,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let mut tiles = Vec::new();
        for row in 0..height {
            for column in 0..width {
                tiles.push(VisualTile {
                    animation_frames: None,
                    column: column as u32,
                    row: row as u32,
                    texture: Handle::default(),
                    priority: false,
                    source: VisualTileSource {
                        tileset_id: Arc::from(object.tileset.as_str()),
                        metatile_id: 0,
                        subtile_column: (column % 4) as u8,
                        subtile_row: (row % 4) as u8,
                        tile_index: object.ground,
                    },
                });
            }
        }
        for y in 0..object.tiles.len() {
            for x in 0..object.tiles[0].len() {
                let source = &mut tiles[y * width + x].source;
                source.metatile_id = object.metatiles.as_ref().map_or(object.metatile, |blocks| {
                    blocks[(usize::from(object.origin[1]) + y) / 4]
                        [(usize::from(object.origin[0]) + x) / 4]
                });
                source.subtile_column = ((usize::from(object.origin[0]) + x) % 4) as u8;
                source.subtile_row = ((usize::from(object.origin[1]) + y) % 4) as u8;
                source.tile_index = object.tiles[y][x];
            }
        }
        (tiles, geometry)
    }
    fn resolve_fixture(
        tiles: &[VisualTile],
        geometry: &GridGeometry,
        profiles: Option<&Document>,
    ) -> Vec<Placement> {
        resolve(
            "UnreviewedPokecenter1F",
            &tiles.iter().collect::<Vec<_>>(),
            geometry,
            profiles,
        )
    }
    #[test]
    fn kanto_native_cafe_trashcan_requires_complete_drawing_and_keeps_footing() {
        let mut object=profile("Shared lab round stool");object.tileset="game_corner".into();
        object.metatile=0x29;object.metatiles=None;object.origin=[0,0];object.ground=1;
        object.tiles=vec![vec![2;4],vec![2;4],vec![1,1,0x46,0x47],vec![1,1,0x56,0x57]];
        let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        let p=resolve("CeladonCafe",&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::TrashCan).unwrap();
        assert_eq!((p.column,p.row,p.width,p.height),(2,2,2,2));
        let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],authored_cells:vec![None;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];assert!(append(&mut mesh,"CeladonCafe",&refs,&g,&p,&mut claimed,[0,0]));
        assert!(mesh.solid.positions.is_empty());assert!(mesh.textured.positions.iter().any(|p|p[1]>7.));
        assert!(mesh.textured.positions.iter().all(|p|p[0]>=16.&&p[0]<=32.&&p[2]>=16.&&p[2]<=32.&&p[1]<=8.+1e-5));
        assert!(p.indices(g.width).all(|i|claimed[i]));assert!(mesh.footing_heights.iter().all(|h|*h==1.25));
        assert!(resolve("OlivineCafe",&refs,&g,None).iter().all(|p|p.kind!=ModelKind::TrashCan));
        tiles[2*g.width+2].source.tile_index=0xffff;
        assert!(resolve("CeladonCafe",&tiles.iter().collect::<Vec<_>>(),&g,None).iter().all(|p|p.kind!=ModelKind::TrashCan));
    }

    #[test]
    fn kanto_native_cafe_uses_shared_complete_display_and_connected_gold_wall() {
        for (block,drawing,kind) in [
            (4,vec![vec![0x16,0x17,0x17,0x18],vec![0x26,0x27,0x27,0x28],vec![0x36,0x37,0x37,0x38]],ModelKind::ArcadePrizeRack),
            (2,vec![vec![2;4],vec![2;4]],ModelKind::ArcadeWall),
            (0x28,vec![vec![2,2,6,7],vec![2,2,8,9]],ModelKind::ArcadeWall),
        ] {
            let mut object=profile("Shared lab round stool");object.tileset="game_corner".into();
            object.metatile=block;object.metatiles=None;object.origin=[0,0];object.ground=1;object.tiles=drawing;
            let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let placements=resolve("CeladonCafe",&refs,&g,None);let p=placements.iter().find(|p|p.kind==kind).unwrap();
            assert_eq!(p.width,4);assert_eq!(p.height,if block==4 {3}else{2});
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],authored_cells:vec![None;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            assert!(append(&mut mesh,"CeladonCafe",&refs,&g,p,&mut claimed,[0,0]));
            assert!(p.indices(g.width).all(|i|claimed[i]));assert!(mesh.footing_heights.iter().all(|h|*h==1.25));
            if kind==ModelKind::ArcadeWall {
                assert!(mesh.solid.positions.is_empty());
                assert!(mesh.textured.positions.iter().all(|p|p[0]>=0.&&p[0]<=32.&&p[1]<=32.));
                assert!(mesh.textured.positions.iter().any(|p|p[1]==32.));
                assert_eq!(mesh.textured.indices.len(),4*2*6+(5+4*4)*6,"one ceiling-height closed wall, no overlapping panels");
            } else {assert!(!mesh.solid.indices.is_empty(),"complete shelf must be a real shared cabinet");}
            tiles[0].source.tile_index=0xffff;
            assert!(resolve("CeladonCafe",&tiles.iter().collect::<Vec<_>>(),&g,None).iter().all(|p|p.kind!=kind));
            let mut other=Vec::new();
            append_cafe_architecture_candidates("OlivineCafe",&tiles.iter().collect::<Vec<_>>(),&g,&mut vec![false;tiles.len()],&mut other);
            assert!(other.is_empty());
        }
    }

    #[test]
    fn kanto_native_house_windows_keep_curtain_source_and_closed_wall_course() {
        for (atlas,block,origin,drawing,map) in [
            ("house",5,[2,0],vec![vec![0x24,0x4a],vec![0x34,0x2c]],"BluesHouse"),
            ("players_house",1,[0,0],vec![vec![0x2c,0x2d],vec![0x11,0x11]],"RedsHouse1F"),
        ] {
            let mut object=profile("Shared lab round stool");object.tileset=atlas.into();
            object.metatile=block;object.metatiles=None;object.origin=origin;object.ground=1;
            object.tiles=drawing;let (mut tiles,g)=fixture(&object);
            let refs=tiles.iter().collect::<Vec<_>>();let placements=resolve(map,&refs,&g,None);
            let p=placements.iter().find(|p|p.kind==ModelKind::WindowWall).unwrap();
            assert!(native_house_window_source(p,&refs,&g));
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],authored_cells:vec![None;tiles.len()],..Default::default()};
            assert!(append(&mut mesh,map,&refs,&g,p,&mut vec![false;tiles.len()],[0,0]));
            assert!(mesh.solid.positions.is_empty(),"aqua substitute cannot remain");
            assert!(mesh.textured.positions.iter().all(|p|p[1]>=0.&&p[1]<=32.));
            assert!(mesh.textured.positions.iter().any(|p|p[1]==32.));
            assert!(mesh.textured.positions.iter().any(|p|p[2]>15.&&p[2]<16.),"curtain relief must remain inside closed wall");
            for ids in mesh.textured.indices[4*6..].chunks_exact(3) {
                let [a,b,c]=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                let normal=Vec3::from_array(mesh.textured.normals[ids[0]as usize]);
                assert!((b-a).cross(c-a).dot(normal)>0.);
            }
            let (u0,_,v0,_)=g.uv(p.column,p.row);
            let (_,u1,_,v1)=g.uv(p.column+1,p.row+1);
            assert!(mesh.textured.uvs.contains(&[u0,v0]));assert!(mesh.textured.uvs.contains(&[u1,v1]));
            assert!(mesh.footing_heights.iter().all(|h|*h==1.25));
            tiles[0].source.tile_index=0xffff;
            assert!(resolve(map,&tiles.iter().collect::<Vec<_>>(),&g,None).iter().all(|p|p.kind!=ModelKind::WindowWall));
        }
    }

    #[test]
    fn kanto_native_televisions_use_own_blue_crt_and_gold_stand_source() {
        for (atlas,block,origin,tileset_tiles,map) in [
            ("house",0x1e,[0,1],vec![vec![6,7],vec![0x16,0x17],vec![0x1e,0x1f]],"BluesHouse"),
            ("players_house",2,[2,1],vec![vec![6,7],vec![0x16,0x17],vec![0x18,0x19]],"RedsHouse2F"),
            ("players_house",0x11,[0,1],vec![vec![6,7],vec![0x16,0x17],vec![8,9]],"RedsHouse1F"),
            ("players_house",0x13,[2,0],vec![vec![6,7],vec![0x16,0x17]],"RedsHouse2F"),
            ("players_room",3,[0,1],vec![vec![0x3b,0x3c],vec![0x4b,0x4c],vec![0x5b,0x5c]],"PlayersHouse2F"),
        ] {
            let mut object=profile("Shared lab round stool");object.tileset=atlas.into();
            object.metatile=block;object.metatiles=None;object.origin=origin;object.ground=if atlas=="players_room" {2}else{1};
            object.tiles=tileset_tiles;let (mut tiles,g)=fixture(&object);
            let refs=tiles.iter().collect::<Vec<_>>();let placements=resolve(map,&refs,&g,None);
            let p=placements.iter().find(|p|p.kind==ModelKind::Television).unwrap();
            let swatches=native_house_television_source(p,&refs,&g).unwrap();
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],authored_cells:vec![None;tiles.len()],..Default::default()};
            assert!(append(&mut mesh,map,&refs,&g,p,&mut vec![false;tiles.len()],[0,0]));
            assert!(mesh.solid.positions.is_empty(),"imported pale wood TV must not remain");
            for (role,pixel) in swatches.into_iter().enumerate() {
                if p.height==2 && role==0 {continue;}
                let uv=g.uv(p.column+pixel[0]/8,p.row+pixel[1]/8);
                let sample=[uv.0+(uv.1-uv.0)*((pixel[0]%8)as f32+0.5)/8.,
                    uv.2+(uv.3-uv.2)*((pixel[1]%8)as f32+0.5)/8.];
                assert!(mesh.textured.uvs[2*p.height*4..].contains(&sample));
            }
            assert!(mesh.footing_heights.iter().all(|h|*h==1.25));
            tiles[0].source.tile_index=0xffff;
            assert!(resolve(map,&tiles.iter().collect::<Vec<_>>(),&g,None).iter().all(|p|p.kind!=ModelKind::Television));
        }
    }

    #[test]
    fn traditional_bookcase_finish_preserves_custom_and_unreviewed_drawings() {
        let drawing=INTERIOR_DRAWINGS.iter().find(|d|d.tileset=="traditional_house"&&d.kind==ModelKind::Bookcase).unwrap();
        let mut object=profile("Shared facility paired bookcases 06");object.name="Custom traditional shelf".into();
        object.tileset=drawing.tileset.into();object.metatile=drawing.block;object.origin=drawing.origin;
        object.metatiles=None;object.map=None;object.maps=None;object.parts.clear();object.ground=0x11;
        object.tiles=drawing.tiles.chunks_exact(2).map(|row|row.to_vec()).collect();
        let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        for map in ["KurtsHouse","EcruteakItemfinderHouse","VioletKylesHouse"] {
            let p=resolve(map,&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::Bookcase).unwrap();
            assert_eq!(native_bookcase_source(map,&p,&refs,&g),Some([[3,10],[3,12]]));
            let mut mesh=TerrainMeshData{footing_heights:vec![2.75;tiles.len()],..Default::default()};
            let mut claims=vec![false;tiles.len()];assert!(append(&mut mesh,map,&refs,&g,&p,&mut claims,[0,0]));
            assert!(mesh.solid.indices.is_empty());assert_eq!(claims.iter().filter(|&&v|v).count(),8);
            assert_eq!(mesh.footing_heights,vec![2.75;tiles.len()]);
            let custom=Document{objects:vec![object.clone()],atmosphere:None};
            assert!(!resolve(map,&refs,&g,Some(&custom)).iter().any(|p|p.kind==ModelKind::Bookcase));
            assert!(native_bookcase_source("UnreviewedTraditionalHouse",&p,&refs,&g).is_none());
        }
    }

    #[test]
    fn source_bookcase_finishes_cover_external_pack_and_keep_claims_and_support() {
        let explicit=std::env::var_os("CRYSTAL_RENDER_TEST_PACK");
        let path=explicit.as_ref().map(std::path::PathBuf::from).unwrap_or_else(||
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        assert!(explicit.is_none()||path.is_file());if !path.is_file(){return;}
        let pack=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let mut maps=std::collections::BTreeMap::new();let mut atlases=std::collections::BTreeSet::new();
        let mut bitmaps=std::collections::BTreeMap::new();
        for (map,module) in &pack.pack().data().maps {
            let atlas=module.attributes.tileset_name.as_str();
            if !native_bookcase_room(map)
                ||!matches!(atlas,"house"|"players_house"|"players_room"|"lab"|"mansion"|"traditional_house"){continue;}
            let layout=&pack.pack().runtime_files()[&format!("data/tilesets/{atlas}_metatiles.bin")];
            let (w,h)=(module.attributes.width as usize*4,module.attributes.height as usize*4);
            let mut tiles=Vec::new();
            for y in 0..h {for x in 0..w {
                let block=module.blocks[y/4*(w/4)+x/4];
                tiles.push(VisualTile{animation_frames:None,column:x as u32,row:y as u32,texture:Default::default(),priority:false,
                    source:VisualTileSource{tileset_id:Arc::from(atlas),metatile_id:block,subtile_column:(x%4)as u8,
                        subtile_row:(y%4)as u8,tile_index:layout[block as usize*16+y%4*4+x%4]as u16}});
            }}
            let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let refs=tiles.iter().collect::<Vec<_>>();
            for p in resolve(map,&refs,&g,None).into_iter().filter(|p|matches!(p.kind,ModelKind::Bookcase|ModelKind::LabBookcase)) {
                let pixels=native_bookcase_source(map,&p,&refs,&g).unwrap_or_else(||panic!("{map}: complete source bookcase missing native finish"));
                *maps.entry(map.clone()).or_insert(0usize)+=1;atlases.insert(atlas.to_string());
                let bitmap=bitmaps.entry(atlas.to_string()).or_insert_with(||
                    image::load_from_memory(&pack.pack().runtime_files()[&format!("gfx/tilesets/{atlas}.png")]).unwrap().to_luma8());
                for unit in 0..p.width/2 {for (role,pixel) in pixels.iter().enumerate() {
                    let source=&refs[(p.row+pixel[1]/8)*g.width+p.column+unit*2+pixel[0]/8].source;
                    let value=bitmap.get_pixel(source.tile_index as u32%16*8+(pixel[0]%8)as u32,
                        source.tile_index as u32/16*8+(pixel[1]%8)as u32).0[0];
                    if role==0 {assert!(matches!(value,85|170),"{map}: cover pixel must be a source middle shade");}
                    else {assert_eq!(value,255,"{map}: page top must use the source light shade");}
                    assert_eq!(pack.pack().data().tilesets[atlas].palette_map[source.tile_index as usize],
                        if atlas=="players_room" {0}else{5},"{map}: books sample their own palette");
                }}
                let mut mesh=TerrainMeshData{footing_heights:vec![2.75;tiles.len()],..Default::default()};
                let mut claims=vec![false;tiles.len()];assert!(append(&mut mesh,map,&refs,&g,&p,&mut claims,[0,0]));
                assert!(mesh.solid.indices.is_empty(),"{map}: pastel books remain");
                for unit in 0..p.width/2 {
                    let selected=crate::interior_models::selected_model(p.kind).unwrap();
                    let mut original=SurfaceMeshData::default();
                    assert!(selected.append_native_bookcase_with_books(&mut original,[0.,16.,0.,8.],0.,29.,[[0.1;2];3],[[0.2;2];2]));
                    let mut adapted=original.clone();
                    append_native_bookcase_base(&mut adapted,&g,&p,&refs,unit,0,0,[0.1;2]);
                    if native_bookcase_closed_base(&p,&refs,&g,unit) {
                        assert_eq!(&adapted.positions[..original.positions.len()],original.positions.as_slice(),"{map}: upper shelf positions");
                        let high=original.positions.iter().map(|v|v[1]).fold(f32::NEG_INFINITY,f32::max);
                        let top=high*0.4;
                        let roles=selected.native_bookcase_structural_mask().unwrap();
                        let mut removed_books=0;
                        for tri in original.indices.chunks_exact(3) {
                            if tri.iter().all(|&i|!roles[i as usize]&&original.positions[i as usize][1]<top-0.001) {
                                assert!(!adapted.indices.chunks_exact(3).any(|t|t==tri),"{map}: lowest books must be removed");
                                removed_books+=1;
                            } else if tri.iter().any(|&i|original.positions[i as usize][1]>=top) {
                                assert!(adapted.indices.chunks_exact(3).any(|t|t==tri),"{map}: upper shelves must remain");
                            }
                        }
                        assert!(removed_books>0);
                        for x in 0..2 {
                            let uv=g.uv(p.column+unit*2+x,p.row+p.height-1);
                            assert!(adapted.uvs[original.positions.len()..].contains(&[uv.0,uv.2]),"{map}: own cupboard/drawer front");
                            assert!(adapted.uvs[original.positions.len()..].contains(&[uv.1,uv.3]));
                        }
                    } else {assert_eq!(adapted,original,"{map}: open shelves must stay unchanged");}
                }
                assert_eq!(claims.iter().filter(|&&v|v).count(),p.width*p.height);
                assert!(mesh.textured.colors.iter().all(|c|c[0]==c[1]&&c[1]==c[2]));
                let mut support=vec![2.75;tiles.len()];
                if let Some(height)=p.footing_pixels {for i in p.indices(w){support[i]=height;}}
                assert_eq!(mesh.footing_heights,support,"{map}: authored support");
                assert!(native_bookcase_source("UnreviewedHouse",&p,&refs,&g).is_none());
                for i in p.indices(w) {
                    let mut changed=tiles.clone();changed[i].source.tile_index=0xffff;
                    assert!(native_bookcase_source(map,&p,&changed.iter().collect::<Vec<_>>(),&g).is_none(),"{map}: damaged source");
                }
            }
        }
        assert!(maps.contains_key("BillsHouse"));assert!(maps.contains_key("PlayersHouse2F"));
        assert!(maps.contains_key("RedsHouse2F"));assert!(maps.contains_key("OaksLab"));
        assert!(maps.contains_key("ElmsLab"));assert!(maps.contains_key("EarlsPokemonAcademy"));
        assert!(maps.contains_key("KurtsHouse"),"timber-floor ground sample must enable the actual traditional shelves");
        println!("Source bookcase atlases: {atlases:?}");
        println!("Source bookcase maps: {}",maps.keys().cloned().collect::<Vec<_>>().join(","));
    }

    #[test]
    fn native_house_cabinet_computers_require_complete_drawings_and_preserve_custom_ownership() {
        for (map,block,origin,drawing) in [
            ("DayCare",0x10,[0,0],vec![vec![0x40,0x41],vec![0x20,0x21],vec![0x42,0x43],vec![0x1e,0x1f]]),
            ("TrainerHouse1F",0x1c,[2,1],vec![vec![0x40,0x41],vec![0x20,0x21],vec![0x42,0x43]]),
        ] {
            let mut object=profile("Shared lab round stool");object.name="Custom PC cabinet".into();
            object.tileset="house".into();object.metatile=block;object.metatiles=None;object.origin=origin;
            object.ground=1;object.tiles=drawing;object.map=None;object.maps=None;object.parts.clear();
            let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let p=resolve(map,&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::Computer).unwrap();
            let swatches=native_computer_source(&p,&refs,&g).unwrap();
            assert_eq!((p.depth_pixels,p.height_pixels),(12.,22.));
            let mut mesh=TerrainMeshData{footing_heights:vec![2.75;tiles.len()],..Default::default()};
            let mut claims=vec![false;tiles.len()];assert!(append(&mut mesh,map,&refs,&g,&p,&mut claims,[0,0]));
            assert!(mesh.solid.indices.is_empty());assert_eq!(claims.iter().filter(|&&v|v).count(),p.width*p.height);
            assert_eq!(mesh.footing_heights,vec![2.75;tiles.len()]);
            let blue=native_source_swatches(&p,&g,[swatches[4]])[0];let mut roof=0;
            for ((v,n),uv) in mesh.textured.positions.iter().zip(&mesh.textured.normals).zip(&mesh.textured.uvs) {
                if (uv[0]-blue[0]).abs()<1e-6&&(uv[1]-blue[1]).abs()<1e-6 {
                    assert!(n[1]>0.5);assert!((v[1]-22.).abs()<1e-4);roof+=1;
                }
            }
            assert!(roof>0,"source blue must remain on the monitor roof");
            let custom=Document{objects:vec![object.clone()],atmosphere:None};
            assert!(!resolve(map,&refs,&g,Some(&custom)).iter().any(|p|p.kind==ModelKind::Computer));
            let mut old=TerrainMeshData::default();
            assert!(append(&mut old,"UnreviewedHouse",&refs,&g,&p,&mut vec![false;tiles.len()],[0,0]));
            assert!(!old.solid.indices.is_empty(),"unreviewed computer geometry stays on its original path");
            drop(refs);
            for i in p.indices(g.width) {
                let before=tiles[i].source.tile_index;tiles[i].source.tile_index=0xffff;
                assert!(!resolve(map,&tiles.iter().collect::<Vec<_>>(),&g,None).iter().any(|p|p.kind==ModelKind::Computer));
                tiles[i].source.tile_index=before;
            }
        }
    }

    #[test]
    fn native_house_electronics_cover_external_pack_and_keep_claims_and_support() {
        let explicit=std::env::var_os("CRYSTAL_RENDER_TEST_PACK");
        let path=explicit.as_ref().map(std::path::PathBuf::from).unwrap_or_else(||
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        assert!(explicit.is_none()||path.is_file());if !path.is_file(){return;}
        let pack=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let mut radios=std::collections::BTreeSet::new();let mut computers=std::collections::BTreeSet::new();
        for (map,module) in &pack.pack().data().maps {
            let atlas=module.attributes.tileset_name.as_str();
            if !reviewed_native_room(map)||!matches!(atlas,"house"|"players_house"|"players_room"){continue;}
            let layout=&pack.pack().runtime_files()[&format!("data/tilesets/{atlas}_metatiles.bin")];
            let (w,h)=(module.attributes.width as usize*4,module.attributes.height as usize*4);
            let mut tiles=Vec::new();
            for y in 0..h {for x in 0..w {
                let block=module.blocks[y/4*(w/4)+x/4];
                tiles.push(VisualTile{animation_frames:None,column:x as u32,row:y as u32,texture:Default::default(),priority:false,
                    source:VisualTileSource{tileset_id:Arc::from(atlas),metatile_id:block,subtile_column:(x%4)as u8,
                        subtile_row:(y%4)as u8,tile_index:layout[block as usize*16+y%4*4+x%4]as u16}});
            }}
            let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let refs=tiles.iter().collect::<Vec<_>>();
            for p in resolve(map,&refs,&g,None) {
                if native_house_radio_source(&p,&refs,&g) {radios.insert(map.clone());}
                else if native_computer_source(&p,&refs,&g).is_some() {computers.insert(map.clone());}
                else {continue;}
                let mut mesh=TerrainMeshData{footing_heights:vec![2.75;tiles.len()],..Default::default()};
                let mut claims=vec![false;tiles.len()];assert!(append(&mut mesh,map,&refs,&g,&p,&mut claims,[0,0]));
                assert!(mesh.solid.indices.is_empty(),"{map}: generic electronics paint remains");
                assert_eq!(claims.iter().filter(|&&v|v).count(),p.width*p.height);
                let mut support=vec![2.75;tiles.len()];
                if let Some(height)=p.footing_pixels {for i in p.indices(w){support[i]=height;}}
                assert_eq!(mesh.footing_heights,support,"{map}: original support");
                if let Some(swatches)=native_computer_source(&p,&refs,&g) {
                    if atlas!="house" {assert_eq!(swatches[3],swatches[1]);assert_eq!(swatches[4],swatches[1]);}
                    for uv in native_source_swatches(&p,&g,swatches) {
                        assert!(mesh.textured.uvs.iter().any(|v|(v[0]-uv[0]).abs()<1e-6&&(v[1]-uv[1]).abs()<1e-6),
                            "{map}: missing source computer swatch {uv:?}");
                    }
                }
            }
        }
        assert!(radios.contains("BluesHouse"));assert!(radios.contains("PlayersNeighborsHouse"));
        assert!(computers.contains("DayCare"));assert!(computers.contains("PlayersHouse2F"));
        println!("Native radio maps: {}",radios.into_iter().collect::<Vec<_>>().join(","));
        println!("Native computer maps: {}",computers.into_iter().collect::<Vec<_>>().join(","));
    }

    #[test]
    fn house_radio_cabinet_uses_complete_stack_and_respects_custom_ownership() {
        let mut object=profile("Shared lab round stool");
        object.name="Custom radio cabinet".into();object.tileset="house".into();
        object.metatile=0x1d;object.metatiles=None;object.origin=[2,1];object.ground=1;
        object.tiles=vec![vec![0x0c,0x0d],vec![0x1c,0x1d],vec![0x1e,0x1f]];
        object.map=None;object.maps=None;object.parts.clear();
        let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        for &map in WOVEN_HOUSE_MAPS {
            let p=resolve(map,&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::Radio).unwrap();
            assert!(native_house_radio_source(&p,&refs,&g));
            assert_eq!((p.depth_pixels,p.height_pixels),(8.,16.));
            let mut mesh=TerrainMeshData{footing_heights:vec![2.75;tiles.len()],..Default::default()};
            let mut claims=vec![false;tiles.len()];assert!(append(&mut mesh,map,&refs,&g,&p,&mut claims,[0,0]));
            assert!(mesh.solid.indices.is_empty());assert_eq!(claims.iter().filter(|&&v|v).count(),6);
            assert_eq!(mesh.footing_heights,vec![2.75;tiles.len()]);
            assert!(mesh.textured.positions.iter().all(|v|v[1]>=0. && v[1]<=16.001));
            let floor=6*4;
            assert!(mesh.textured.positions[floor..].iter().any(|v|v[1]<8.));
            assert!(mesh.textured.positions[floor..].iter().any(|v|v[1]>8.));
            for uv in native_source_swatches(&p,&g,[[1,16],[4,19],[0,0],[4,12]]) {
                assert!(mesh.textured.uvs[floor..].contains(&uv));
            }
            let custom=Document{objects:vec![object.clone()],atmosphere:None};
            assert!(!resolve(map,&refs,&g,Some(&custom)).iter().any(|p|p.kind==ModelKind::Radio));
        }
        let old=resolve("UnreviewedHouse",&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::Radio).unwrap();
        assert_eq!(old.height_pixels,12.);
        drop(refs);tiles[0].source.tile_index=0xffff;
        for &map in WOVEN_HOUSE_MAPS {
            assert!(!resolve(map,&tiles.iter().collect::<Vec<_>>(),&g,None).iter().any(|p|p.kind==ModelKind::Radio));
        }
    }

    #[test]
    fn johto_and_kanto_house_finishes_use_complete_source_drawings_and_custom_ownership() {
        for (kind,block,origin,drawing) in [
            (ModelKind::Stool,0x0c,[0,0],vec![vec![2,3],vec![0x12,0x13]]),
            (ModelKind::WindowWall,5,[2,0],vec![vec![0x24,0x4a],vec![0x34,0x2c]]),
            (ModelKind::Television,0x1e,[0,1],vec![vec![6,7],vec![0x16,0x17],vec![0x1e,0x1f]]),
        ] {
            let mut object=profile("Shared lab round stool");
            object.name="Custom house drawing".into();object.tileset="house".into();
            object.metatile=block;object.metatiles=None;object.origin=origin;object.ground=1;
            object.tiles=drawing;object.map=None;object.maps=None;object.parts.clear();
            let (mut tiles,g)=fixture(&object);
            let refs=tiles.iter().collect::<Vec<_>>();
            for &map in WOVEN_HOUSE_MAPS {
                let p=resolve(map,&refs,&g,None).into_iter().find(|p|p.kind==kind)
                    .unwrap_or_else(||panic!("{map}: complete {kind:?} must resolve"));
                let mut mesh=TerrainMeshData{footing_heights:vec![2.75;tiles.len()],..Default::default()};
                let mut claims=vec![false;tiles.len()];
                assert!(append(&mut mesh,map,&refs,&g,&p,&mut claims,[0,0]));
                assert!(mesh.solid.indices.is_empty(),"{map}: generic {kind:?} paint remains");
                assert!(!mesh.textured.indices.is_empty());
                assert_eq!(claims.iter().filter(|&&v|v).count(),p.width*p.height);
                assert_eq!(mesh.footing_heights,vec![2.75;tiles.len()]);
                let custom=Document{objects:vec![object.clone()],atmosphere:None};
                assert!(!resolve(map,&refs,&g,Some(&custom)).iter().any(|p|p.kind==kind),
                    "{map}: custom drawing must retain ownership");
            }
            drop(refs);tiles[0].source.tile_index=0xffff;
            for &map in WOVEN_HOUSE_MAPS {
                assert!(!resolve(map,&tiles.iter().collect::<Vec<_>>(),&g,None)
                    .iter().any(|p|p.kind==kind),"{map}: damaged drawing must not resolve");
            }
        }
    }

    fn assert_native_tabletop_projection(mesh:&SurfaceMeshData,g:&GridGeometry,p:&Placement) {
        let first=p.width*p.height*4;
        let positions=&mesh.positions[first..];
        let w=positions.iter().map(|v|v[0]).fold(f32::INFINITY,f32::min);
        let e=positions.iter().map(|v|v[0]).fold(f32::NEG_INFINITY,f32::max);
        let n=positions.iter().map(|v|v[2]).fold(f32::INFINITY,f32::min);
        let s=positions.iter().map(|v|v[2]).fold(f32::NEG_INFINITY,f32::max);
        let tw=(e-w)/p.width as f32;let th=(s-n)/(p.height-1)as f32;
        let mut seen=std::collections::BTreeSet::new();
        for ids in mesh.indices.chunks_exact(3).filter(|ids|ids.iter().all(|&i| {
            let i=i as usize;i>=first&&mesh.normals[i][1]>0.999
                &&mesh.positions[i][1]>p.height_pixels*g.tile_height/8.*0.95
                &&mesh.positions[i][1]<=p.height_pixels*g.tile_height/8.+0.001
        })) {
            let center=ids.iter().fold(Vec3::ZERO,|a,&i|a+Vec3::from_array(mesh.positions[i as usize]))/3.;
            let x=((center.x-w)/tw).floor()as usize;let y=((center.z-n)/th).floor()as usize;
            assert!(x<p.width&&y<p.height-1,"apron cannot enter the native top");
            seen.insert((x,y));let uv=g.uv(p.column+x,p.row+y);
            for &i in ids {
                let v=mesh.positions[i as usize];
                let expected=if p.width==8&&(3..5).contains(&x)&&(2..5).contains(&y) {
                    native_source_swatches(p,g,[[17,9]])[0]
                } else {[uv.0+(uv.1-uv.0)*(v[0]-w-x as f32*tw)/tw,
                    uv.2+(uv.3-uv.2)*(v[2]-n-y as f32*th)/th]};
                for axis in 0..2 {assert!((mesh.uvs[i as usize][axis]-expected[axis]).abs()<1e-5);}
            }
        }
        assert_eq!(seen.len(),p.width*(p.height-1));
    }

    #[test]
    fn native_house_table_reveal_owns_each_complete_table_without_floor_or_support() {
        let explicit=std::env::var_os("CRYSTAL_RENDER_TEST_PACK");
        let path=explicit.as_ref().map(std::path::PathBuf::from).unwrap_or_else(||
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        assert!(explicit.is_none()||path.is_file(),"explicit external pack must exist");
        if !path.is_file(){return;}
        let pack=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let mut maps=std::collections::BTreeMap::new();
        for (map,module) in &pack.pack().data().maps {
            let tileset=module.attributes.tileset_name.as_str();
            if !matches!(tileset,"house"|"players_house"|"players_room"|"lab"|"mansion"){continue;}
            let layout=&pack.pack().runtime_files()[&format!("data/tilesets/{tileset}_metatiles.bin")];
            let (w,h)=(module.attributes.width as usize*4,module.attributes.height as usize*4);
            let mut tiles=Vec::new();
            for y in 0..h {for x in 0..w {
                let block=module.blocks[y/4*(w/4)+x/4];
                tiles.push(VisualTile{animation_frames:None,column:x as u32,row:y as u32,texture:Default::default(),priority:false,
                    source:VisualTileSource{tileset_id:Arc::from(tileset),metatile_id:block,subtile_column:(x%4)as u8,
                        subtile_row:(y%4)as u8,tile_index:layout[block as usize*16+y%4*4+x%4]as u16}});
            }}
            let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let refs=tiles.iter().collect::<Vec<_>>();
            for p in resolve(map,&refs,&g,None) {
                if !native_house_dining_table(map,&refs,&g,&p){continue;}
                *maps.entry(map.clone()).or_insert(0usize)+=1;
                let tabletop=(p.base_pixels+p.height_pixels)*g.tile_height/8.;
                for y in p.row..p.row+p.height {for x in p.column..p.column+p.width {
                    let expected=(y<p.row+native_dining_table_rows(&p)).then_some(tabletop);
                    assert_eq!(traditional_tabletop_height(map,&refs,&g,None,x,y),expected,
                        "{map}: stationary decorations must rest on the actual table");
                }}
                assert_eq!(traditional_tabletop_height(map,&refs,&g,None,g.width,g.height),None);
                let mut mesh=TerrainMeshData{footing_heights:vec![2.75;tiles.len()],..Default::default()};
                let mut claims=vec![false;tiles.len()];assert!(append(&mut mesh,map,&refs,&g,&p,&mut claims,[0,0]));
                assert_eq!(mesh.textured.cutaway_ranges,vec![p.width*p.height*4..mesh.textured.positions.len()],"{map}");
                assert!(mesh.depth_coverage_textured_ranges.is_empty(),"{map}: tables use smooth transparency");
                assert_eq!(mesh.layered_alpha_textured_ranges,if p.width==8 {
                    mesh.textured.cutaway_ranges.clone()
                }else{Vec::new()},"{map}: only the layered centerpiece needs fine sorting");
                assert!(mesh.textured.positions[..p.width*p.height*4].iter().all(|v|v[1]==0.));
                assert_eq!(claims.iter().filter(|&&v|v).count(),p.width*p.height);
                let mut expected=vec![2.75;tiles.len()];
                if let Some(height)=p.footing_pixels {
                    for i in p.indices(w) {expected[i]=height*g.tile_height/8.;}
                }
                assert_eq!(mesh.footing_heights,expected,"{map}: authored support remains authoritative");
                assert_native_tabletop_projection(&mesh.textured,&g,&p);
            }
        }
        assert_eq!(maps.get("PlayersHouse1F"),Some(&1));assert_eq!(maps.get("PlayersHouse2F"),Some(&1));
        assert_eq!(maps.get("DayOfWeekSiblingsHouse"),Some(&1));
        assert!(maps.len()>10);
        println!("Native table reveal maps: {}",maps.keys().cloned().collect::<Vec<_>>().join(","));
    }

    #[test]
    fn johto_player_home_tables_use_complete_source_and_preserve_footing() {
        for (name,map) in [("PlayersHouse1F dining table","PlayersHouse1F"),
            ("Player bedroom table","PlayersHouse2F")] {
            let object=profile(name);let (mut tiles,g)=fixture(&object);
            let refs=tiles.iter().collect::<Vec<_>>();
            let placements=resolve(map,&refs,&g,None);
            let p=placements.iter().find(|p|p.kind==ModelKind::DiningTable).unwrap();
            assert!(native_house_dining_table(map,&refs,&g,p));
            if map=="PlayersHouse2F" {assert_eq!(p.depth_pixels,16.);}
            let mut mesh=TerrainMeshData{footing_heights:vec![2.5;tiles.len()],
                authored_cells:vec![None;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            assert!(append(&mut mesh,map,&refs,&g,p,&mut claimed,[0,0]));
            assert!(mesh.solid.positions.is_empty(),"generic pink table must be replaced");
            assert!(p.indices(g.width).all(|i|claimed[i]));
            assert!(mesh.footing_heights.iter().all(|h|*h==2.5));
            let table_start=p.width*p.height*4;
            assert_eq!(mesh.textured.cutaway_ranges,vec![table_start..mesh.textured.positions.len()]);
            assert!(mesh.depth_coverage_textured_ranges.is_empty(),"tables use smooth transparency");
            assert!(mesh.textured.positions[..table_start].iter().all(|v|v[1]==0.));
            if map=="PlayersHouse2F" {
                let table=&mesh.textured.positions[p.width*p.height*4..];
                assert!(!table.is_empty());
                assert!(table.iter().all(|v|v[2]>=p.row as f32*8.-0.001
                    && v[2]<=(p.row+2)as f32*8.+0.001),"table cannot extend into the player approach tile");
            }
            tiles[0].source.tile_index=0xffff;
            assert!(resolve(map,&tiles.iter().collect::<Vec<_>>(),&g,None)
                .iter().all(|p|p.kind!=ModelKind::DiningTable));
        }
    }

    #[test]
    fn kanto_native_red_dining_table_claims_complete_feet_and_keeps_square_top() {
        let object=profile("PlayersHouse1F dining table");let (mut tiles,g)=fixture(&object);
        let refs=tiles.iter().collect::<Vec<_>>();let placements=resolve("RedsHouse1F",&refs,&g,None);
        let p=placements.iter().find(|p|p.kind==ModelKind::DiningTable).unwrap();
        assert_eq!((p.width,p.height),(4,5));
        let mut mesh=TerrainMeshData{footing_heights:vec![2.5;tiles.len()],authored_cells:vec![None;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        assert!(append(&mut mesh,"RedsHouse1F",&refs,&g,p,&mut claimed,[0,0]));
        assert!(p.indices(g.width).all(|i|claimed[i]));
        assert!(mesh.solid.positions.is_empty());
        let vertices=&mesh.textured.positions[4*5*4..];
        let min=vertices.iter().fold([f32::INFINITY;3],|a,p|std::array::from_fn(|i|a[i].min(p[i])));
        let max=vertices.iter().fold([f32::NEG_INFINITY;3],|a,p|std::array::from_fn(|i|a[i].max(p[i])));
        assert!((max[2]-min[2]-32.).abs()<0.001,"source feet cannot elongate the physical top");
        assert!(max[2]<=32. && min[2]>=0.);
        assert!(mesh.footing_heights.iter().all(|h|*h==2.5));
        tiles[4*g.width].source.tile_index=0xffff;
        assert!(resolve("RedsHouse1F",&tiles.iter().collect::<Vec<_>>(),&g,None).iter().all(|p|p.kind!=ModelKind::DiningTable));
        let mut clipped=g.clone();clipped.height=4;
        assert!(!native_player_dining_table_complete(&tiles.iter().collect::<Vec<_>>(),&clipped,0,0));
    }

    #[test]
    fn kanto_native_dining_table_uses_shared_model_scale_instead_of_legacy_extrusion() {
        let object=profile("Neighbor dining table");let (mut tiles,g)=fixture(&object);
        let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve("BluesHouse",&refs,&g,None);
        let p=placements.iter().find(|p|p.kind==ModelKind::DiningTable).unwrap();
        assert_eq!((p.width,p.height),(4,4));assert_eq!(p.height_pixels,10.);
        let mut mesh=TerrainMeshData{footing_heights:vec![2.5;tiles.len()],authored_cells:vec![None;tiles.len()],..Default::default()};
        assert!(append(&mut mesh,"BluesHouse",&refs,&g,p,&mut vec![false;tiles.len()],[0,0]));
        let table_vertices=&mesh.textured.positions[4*4*4..];
        let min=table_vertices.iter().fold([f32::INFINITY;3],|a,p|std::array::from_fn(|i|a[i].min(p[i])));
        let max=table_vertices.iter().fold([f32::NEG_INFINITY;3],|a,p|std::array::from_fn(|i|a[i].max(p[i])));
        assert!((max[1]-min[1]-10.).abs()<0.001);
        assert!((max[0]-min[0]).min(max[2]-min[2])>31.,"complete table cannot collapse into a narrow strip");
        assert!(mesh.footing_heights.iter().all(|h|*h==2.5));
        assert!(mesh.solid.positions.is_empty(),"pink imported finish cannot remain beside native geometry");
        let uv=g.uv(p.column,p.row);
        for pixel in [[7,2],[5,2]] {
            let sample=[uv.0+(uv.1-uv.0)*(pixel[0] as f32+0.5)/8.,
                uv.2+(uv.3-uv.2)*(pixel[1] as f32+0.5)/8.];
            assert!(mesh.textured.uvs[4*4*4..].contains(&sample));
        }
        tiles[0].source.tile_index=0xffff;
        assert!(resolve("BluesHouse",&tiles.iter().collect::<Vec<_>>(),&g,None).iter()
            .all(|p|p.kind!=ModelKind::DiningTable),"altered table must remain source art");
    }

    #[test]
    fn kanto_native_player_wall_fragments_join_course_without_claiming_pc_or_stairs() {
        for block in [0x0b,0x10] {
            let mut object=profile("Shared lab round stool");
            object.tileset="players_house".into();object.metatile=block;
            object.metatiles=None;object.origin=[0,0];object.ground=crate::players_house::FLOOR_TILE;
            object.tiles=if block==0x10 {vec![vec![0x11;4],vec![0x20,0x21,0x11,0x11],
                vec![0x30,0x31,1,1],vec![0x0c,0x0d,1,1]]}
                else {vec![vec![0x11,0x11,0x50,0x51],vec![0x11,0x11,0x60,0x61],vec![1;4],vec![1;4]]};
            let (mut tiles,g)=fixture(&object);
            let mut claimed=vec![false;tiles.len()];let mut placements=Vec::new();
            append_player_house_wall_fragments("RedsHouse2F",&tiles.iter().collect::<Vec<_>>(),&g,&mut claimed,&mut placements);
            assert_eq!(placements.len(),if block==0x10 {2}else{1});
            assert_eq!(claimed.iter().filter(|c|**c).count(),if block==0x10 {6}else{4});
            let mut mesh=TerrainMeshData{footing_heights:vec![2.5;tiles.len()],authored_cells:vec![None;tiles.len()],..Default::default()};
            let mut rendered=vec![false;tiles.len()];
            for p in &placements {
                assert!(append(&mut mesh,"RedsHouse2F",&tiles.iter().collect::<Vec<_>>(),&g,p,&mut rendered,[0,0]));
            }
            assert!(mesh.solid.indices.is_empty());
            assert!(mesh.textured.positions.iter().any(|v|v[1]==16. && v[2]==16.));
            assert!(mesh.footing_heights.iter().all(|h|*h==2.5));
            assert!(!claimed[2*g.width],"PC and staircase cells remain independently owned");
            let p=&placements[0];tiles[p.row*g.width+p.column].source.tile_index=0xffff;
            let mut altered=Vec::new();
            append_player_house_wall_fragments("RedsHouse2F",&tiles.iter().collect::<Vec<_>>(),&g,&mut vec![false;tiles.len()],&mut altered);
            assert_eq!(altered.len(),placements.len()-1,"altered source cannot acquire a wall");
            let mut johto=Vec::new();
            append_player_house_wall_fragments("ElmsHouse",&tiles.iter().collect::<Vec<_>>(),&g,&mut vec![false;tiles.len()],&mut johto);
            assert!(johto.is_empty());
        }
    }

    #[test]
    fn external_mansion_corridor_furniture_reveal_owns_whole_objects_but_not_floor_or_neighbors() {
        let explicit=std::env::var_os("CRYSTAL_RENDER_TEST_PACK");
        let path=explicit.as_ref().map(std::path::PathBuf::from).unwrap_or_else(||
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        assert!(explicit.is_none()||path.is_file());if !path.is_file(){return;}
        let pack=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let files=pack.pack().runtime_files();let layout=&files["data/tilesets/mansion_metatiles.bin"];
        let png=image::load_from_memory(&files["gfx/tilesets/mansion.png"]).unwrap().to_luma8();
        let palette=&pack.pack().data().tilesets["mansion"].palette_map;
        let mut counts=[0usize;3];
        for map in ["CeladonMansion1F","CeladonMansion2F","CeladonMansion3F"] {
            let module=&pack.pack().data().maps[map];
            let (w,h)=(module.attributes.width as usize*4,module.attributes.height as usize*4);
            let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let tiles=(0..w*h).map(|i| {let x=i%w;let y=i/w;
                let block=module.blocks[y/4*(w/4)+x/4];
                VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                    source:VisualTileSource{tileset_id:"mansion".into(),metatile_id:block,
                        subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,
                        tile_index:layout[block as usize*16+y%4*4+x%4]as u16}}
            }).collect::<Vec<_>>();let refs=tiles.iter().collect::<Vec<_>>();
            for p in resolve(map,&refs,&g,None) {
                let owns=native_mansion_corridor_furniture(map,&p,&refs,&g);
                let mut mesh=TerrainMeshData{footing_heights:vec![2.75;w*h],..Default::default()};
                let mut claims=vec![false;w*h];
                assert!(append(&mut mesh,map,&refs,&g,&p,&mut claims,[0,0]));
                if !owns {
                    if !native_house_dining_table(map,&refs,&g,&p) {
                        assert!(mesh.depth_coverage_textured_ranges.is_empty(),"neighbor style unchanged");
                    }
                    continue;
                }
                let kind=match p.kind {ModelKind::Bookcase=>0,ModelKind::LabWorkstation=>1,
                    ModelKind::PendulumClock=>2,_=>unreachable!()};counts[kind]+=1;
                assert!(!native_mansion_corridor_furniture("MobileTradeRoom",&p,&refs,&g));
                assert_eq!(refs[p.ground].source.tile_index,0x11,
                    "revealed furnishings keep the native pink room backing");
                let floor=p.width*p.height*4;
                let range=floor..mesh.textured.positions.len();
                assert_eq!(mesh.textured.cutaway_ranges,vec![range.clone()]);
                assert_eq!(mesh.depth_coverage_textured_ranges,vec![range]);
                assert!(mesh.layered_alpha_textured_ranges.is_empty(),"retain cabinet coverage reveal");
                assert!(mesh.solid.indices.is_empty());
                assert_eq!(claims.iter().filter(|&&c|c).count(),p.width*p.height);
                assert_eq!(mesh.footing_heights,vec![2.75;w*h]);
                if p.kind==ModelKind::PendulumClock {
                    let paint=native_source_swatches(&p,&g,MANSION_CLOCK_PIXELS);
                    assert!(mesh.textured.uvs[floor..].iter().all(|uv|paint.contains(uv)));
                    let mut original=SurfaceMeshData::default();
                    model(ModelKind::PendulumClock).append_fitted(&mut original,[0.,16.,0.,7.],0.,30.);
                    assert_eq!(mesh.textured.indices.len(),original.indices.len()+p.width*p.height*6);
                    for (pixel,value) in MANSION_CLOCK_PIXELS.into_iter().zip([85,170,255,0]) {
                        let raw=refs[(p.row+pixel[1]/8)*w+p.column+pixel[0]/8].source.tile_index as usize;
                        assert_eq!(palette[raw],5);
                        assert_eq!(png.get_pixel((raw%16*8+pixel[0]%8)as u32,
                            (raw/16*8+pixel[1]%8)as u32).0[0],value);
                    }
                }
                let (opaque,_,groups,solid_groups)=crate::occluder_fade::prepare_linked(
                    mesh.textured,mesh.solid,Vec::new());
                assert_eq!(groups.len(),1,"complete furniture is one reveal owner");
                assert!(solid_groups.is_empty());
                assert_eq!(opaque.positions.len(),floor,"all source floor stays opaque");
                assert_eq!(opaque.indices.len(),p.width*p.height*6);
                for i in p.indices(w) {
                    let mut changed=tiles.clone();changed[i].source.tile_index=0xffff;
                    assert!(!native_mansion_corridor_furniture(map,&p,&changed.iter().collect::<Vec<_>>(),&g));
                }
            }
        }
        assert_eq!(counts,[4,4,1],"cover every native Mansion bookcase/workstation/clock");
    }

    #[test]
    fn external_mansion_plant_source_anchors_use_native_green_and_gold_palettes() {
        let explicit=std::env::var_os("CRYSTAL_RENDER_TEST_PACK");
        let path=explicit.as_ref().map(std::path::PathBuf::from).unwrap_or_else(||
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        assert!(explicit.is_none()||path.is_file());if !path.is_file(){return;}
        let pack=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let files=pack.pack().runtime_files();
        let layout=&files["data/tilesets/mansion_metatiles.bin"];
        let png=image::load_from_memory(&files["gfx/tilesets/mansion.png"]).unwrap().to_luma8();
        let palette=&pack.pack().data().tilesets["mansion"].palette_map;
        let mut counts=std::collections::BTreeMap::new();
        for (map,module) in &pack.pack().data().maps {
            if module.attributes.tileset_name!="mansion"{continue;}
            let (w,h)=(module.attributes.width as usize*4,module.attributes.height as usize*4);
            let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let tiles=(0..w*h).map(|i| {let x=i%w;let y=i/w;
                let block=module.blocks[y/4*(w/4)+x/4];
                VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                    source:VisualTileSource{tileset_id:"mansion".into(),metatile_id:block,
                        subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,
                        tile_index:layout[block as usize*16+y%4*4+x%4]as u16}}
            }).collect::<Vec<_>>();let refs=tiles.iter().collect::<Vec<_>>();
            for p in resolve(map,&refs,&g,None).into_iter().filter(|p|native_mansion_plant(map,p,&refs,&g)) {
                *counts.entry(map.clone()).or_insert(0usize)+=1;
                for (role,pixel) in MANSION_PLANT_PIXELS.into_iter().enumerate() {
                    let raw=refs[(p.row+pixel[1]/8)*w+p.column+pixel[0]/8].source.tile_index as usize;
                    assert_eq!(palette[raw],if role<3 {2}else{5});
                    assert_eq!(png.get_pixel((raw%16*8+pixel[0]%8)as u32,
                        (raw/16*8+pixel[1]%8)as u32).0[0],[170,85,0,85,170,0][role]);
                }
            }
        }
        assert_eq!(counts.len(),2);
        for map in ["CeladonMansion1F","CeladonMansion2F"] {assert_eq!(counts.get(map),Some(&1));}
    }

    #[test]
    fn mansion_plants_are_compact_dense_native_green_and_gold_with_exact_ownership() {
        let mut object=profile("Shared lab round stool");
        object.name="Source mansion plant".into();object.tileset="mansion".into();
        object.metatile=0x11;object.metatiles=None;object.origin=[2,2];object.ground=0x11;
        object.map=None;object.maps=None;object.parts.clear();
        object.tiles=vec![vec![0x2e,0x2f],vec![0x5e,0x5f]];
        let (tiles,g)=fixture(&object);
        for map in ["CeladonMansion1F","CeladonMansion2F"] {
            let refs=tiles.iter().collect::<Vec<_>>();
            let plant=resolve(map,&refs,&g,None).into_iter().find(|p|native_mansion_plant(map,p,&refs,&g)).unwrap();
            let mut mesh=TerrainMeshData{footing_heights:vec![1.75;tiles.len()],..Default::default()};
            let mut claims=vec![false;tiles.len()];
            assert!(append(&mut mesh,map,&refs,&g,&plant,&mut claims,[0,0]));
            assert!(mesh.solid.indices.is_empty());
            assert_eq!(claims.iter().filter(|&&v|v).count(),4);
            assert!(mesh.footing_heights.iter().all(|h|*h==1.75));
            assert!(mesh.textured.positions.iter().all(|v|v[1]>=0.&&v[1]<=16.001));
            assert!(mesh.textured.positions.iter().any(|v|v[1]>15.));
            let paint=native_source_swatches(&plant,&g,MANSION_PLANT_PIXELS);
            // Skip the four source floor quads; every plant face uses own-source
            // sampled pigments and no pale imported material survives.
            assert!(mesh.textured.uvs[16..].iter().all(|uv|paint.contains(uv)));
            let mut original=SurfaceMeshData::default();
            model(ModelKind::PottedPlant).append_fitted(&mut original,[0.,8.,0.,8.],0.,16.);
            assert!(mesh.textured.indices.len()>original.indices.len()+24,
                "dense rotated foliage clusters replace the sparse imported crown");
            assert!(!native_mansion_plant("CeladonMansion3F",&plant,&refs,&g));
            for index in plant.indices(g.width) {
                let mut damaged=tiles.clone();damaged[index].source.tile_index^=1;
                assert!(!native_mansion_plant(map,&plant,&damaged.iter().collect::<Vec<_>>(),&g));
            }
        }
    }

    #[test]
    fn johto_player_home_stools_use_complete_source_drawing_and_preserve_footing() {
        let mut object=profile("Shared lab round stool");
        object.name="Player home stool".into();object.tileset="players_house".into();
        object.metatile=0x0c;object.metatiles=None;object.origin=[0,0];object.ground=1;
        object.map=None;object.maps=None;object.parts.clear();
        object.tiles=vec![vec![2,3],vec![0x12,0x13]];
        let (mut tiles,g)=fixture(&object);
        let refs=tiles.iter().collect::<Vec<_>>();
        let seat=resolve("PlayersHouse1F",&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::Stool).unwrap();
        let mut mesh=TerrainMeshData{footing_heights:vec![1.75;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        assert!(append(&mut mesh,"PlayersHouse1F",&refs,&g,&seat,&mut claimed,[0,0]));
        assert!(mesh.solid.indices.is_empty());
        assert_eq!(claimed.iter().filter(|&&v|v).count(),4);
        assert!(mesh.footing_heights.iter().all(|h|*h==1.75));
        assert!(mesh.textured.positions.iter().all(|p|p[1]>=0. && p[1]<=5.));
        let top_vertices=mesh.textured.positions.iter().filter(|v|v[1]==5.).collect::<Vec<_>>();
        let cx=top_vertices.iter().map(|v|v[0]).sum::<f32>()/top_vertices.len() as f32;
        let cz=top_vertices.iter().map(|v|v[2]).sum::<f32>()/top_vertices.len() as f32;
        let rim=top_vertices.iter().map(|v|((v[0]-cx).powi(2)+(v[2]-cz).powi(2)).sqrt())
            .filter(|r|*r>0.01).collect::<Vec<_>>();
        let radius=rim.iter().sum::<f32>()/rim.len() as f32;
        assert!(radius>2. && radius<4. && rim.len()>32
            && rim.iter().all(|r|(*r-radius).abs()<0.01),
            "home stool has a circular beveled wood seat, not rectangular upholstery");
        drop(refs);tiles[0].source.tile_index^=1;
        assert!(!resolve("PlayersHouse1F",&tiles.iter().collect::<Vec<_>>(),&g,None).iter().any(|p|p.kind==ModelKind::Stool));
    }

    #[test]
    fn kanto_native_house_stools_keep_rectangular_source_seat_and_low_authored_height() {
        let mut object=profile("Shared lab round stool");
        object.name="Source house stool".into();object.tileset="house".into();
        object.metatile=0x0c;object.metatiles=None;object.origin=[0,0];object.ground=1;
        object.map=None;object.maps=None;object.parts.clear();
        object.tiles=vec![vec![2,3],vec![0x12,0x13]];
        let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        for map in ["BillsHouse","TrainerHouse1F"] {
            let placements=resolve(map,&refs,&g,None);
            let seat=placements.iter().find(|p|p.kind==ModelKind::Stool).unwrap();
            let mut mesh=TerrainMeshData{footing_heights:vec![1.75;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            assert!(append(&mut mesh,map,&refs,&g,seat,&mut claimed,[0,0]));
            assert!(mesh.solid.indices.is_empty(),"native stool must not retain coral upholstery");
            assert_eq!(claimed.iter().filter(|&&v|v).count(),4);
            assert!(mesh.footing_heights.iter().all(|h|*h==1.75));
            assert!(mesh.textured.positions.iter().all(|p|p[1]>=0. && p[1]<=5.));
            let top=mesh.textured.indices.chunks_exact(3).filter(|tri|
                tri.iter().all(|i|mesh.textured.positions[*i as usize][1]==5.)).map(|tri| {
                let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                (points[1]-points[0]).cross(points[2]-points[0]).y*0.5
            }).sum::<f32>();
            assert!((top-8.*6.).abs()<0.01,"seat must close its compact rectangular footprint");
            for x in 0..2 {let uv=g.uv(seat.column+x,seat.row);
                assert!(mesh.textured.uvs.iter().any(|p|p[0]>=uv.0 && p[0]<=uv.1
                    && p[1]==uv.2+(uv.3-uv.2)*7./8.));
            }
        }
        let seat=resolve("BillsHouse",&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::Stool).unwrap();
        let mut other=TerrainMeshData::default();
        assert!(append(&mut other,"EcruteakHouse",&refs,&g,&seat,&mut vec![false;tiles.len()],[0,0]));
        assert!(!other.solid.indices.is_empty(),"unrelated stool styling remains untouched");
        let custom=Document{objects:vec![object.clone()],atmosphere:None};
        assert!(!resolve("BillsHouse",&refs,&g,Some(&custom)).iter().any(|p|p.kind==ModelKind::Stool));
        drop(refs);tiles[0].source.tile_index^=1;
        assert!(!resolve("BillsHouse",&tiles.iter().collect::<Vec<_>>(),&g,None)
            .iter().any(|p|p.kind==ModelKind::Stool));
    }

    #[test]
    fn kanto_native_soul_house_memorials_keep_own_gold_palette_and_separate_claims() {
        for name in ["Soul House memorial 28 0","Soul House memorial 28 2",
            "Soul House memorial 2a 0","Soul House memorial 2a 2",
            "Soul House memorial 2b 0","Soul House memorial 2b 2"] {
            let object=profile(name);let (mut tiles,g)=fixture(&object);
            let refs=tiles.iter().collect::<Vec<_>>();
            let placements=resolve("SoulHouse",&refs,&g,Some(built_in_profiles()));
            let memorial=placements.iter().find(|p|p.kind==ModelKind::Memorial).unwrap();
            assert_eq!((memorial.width,memorial.height),(2,2));
            let mut mesh=TerrainMeshData{footing_heights:vec![2.5;tiles.len()],..Default::default()};
            let mut claims=vec![false;tiles.len()];
            assert!(append(&mut mesh,"SoulHouse",&refs,&g,memorial,&mut claims,[0,0]));
            assert!(mesh.solid.indices.is_empty());
            assert!(mesh.footing_heights.iter().all(|h|*h==2.5));
            assert_eq!(claims.iter().filter(|&&c|c).count(),4);
            let uv=g.uv(memorial.column,memorial.row);
            for pixel in [[1,1],[4,3],[4,1]] {
                assert!(mesh.textured.uvs.contains(&[uv.0+(uv.1-uv.0)*(pixel[0] as f32+0.5)/8.,
                    uv.2+(uv.3-uv.2)*(pixel[1] as f32+0.5)/8.]));
            }
            let mut other=TerrainMeshData::default();
            assert!(append(&mut other,"OtherHouse",&refs,&g,memorial,
                &mut vec![false;tiles.len()],[0,0]));
            let ground_vertices=other.textured.positions.len();
            let ground_indices=other.textured.indices.len();
            assert_eq!(mesh.textured.positions[..ground_vertices],other.textured.positions);
            assert_eq!(mesh.textured.positions[ground_vertices..],other.solid.positions);
            assert_eq!(mesh.textured.normals[ground_vertices..],other.solid.normals);
            assert_eq!(mesh.textured.indices[ground_indices..].iter()
                .map(|i|*i-ground_vertices as u32).collect::<Vec<_>>(),other.solid.indices);
            let mut changed=object.clone();changed.name="Custom memorial".into();
            let custom=Document{objects:vec![changed],atmosphere:None};
            assert!(!resolve("SoulHouse",&refs,&g,Some(&custom)).iter().any(|p|p.kind==ModelKind::Memorial));
            drop(refs);tiles[0].source.tile_index^=1;
            assert!(!resolve("SoulHouse",&tiles.iter().collect::<Vec<_>>(),&g,None)
                .iter().any(|p|p.kind==ModelKind::Memorial));
        }
    }

    #[test]
    fn johto_standard_marts_keep_source_displays_and_unchanged_claims() {
        for map in ["AzaleaMart","BlackthornMart","CherrygroveMart","EcruteakMart","OlivineMart","VioletMart"] {
            for name in ["Shared Mart Ecruteak Mart refrigerator 0","Shared Mart Ecruteak Mart shelf 0"] {
                let object=profile(name);let (mut tiles,g)=fixture(&object);
                let refs=tiles.iter().collect::<Vec<_>>();
                let placements=resolve(map,&refs,&g,None);
                let p=placements.iter().find(|p|matches!(p.kind,ModelKind::RetailRefrigerator|ModelKind::RetailShelf)).unwrap();
                let mut mesh=TerrainMeshData{footing_heights:vec![2.5;tiles.len()],..Default::default()};
                let mut claimed=vec![false;tiles.len()];
                assert!(append(&mut mesh,map,&refs,&g,p,&mut claimed,[0,0]));
                assert!(mesh.solid.indices.is_empty(),"{map}: source merchandise replaces pastel books");
                assert_eq!(claimed.iter().filter(|&&v|v).count(),8);
                assert!(mesh.footing_heights.iter().all(|h|*h==2.5));
                for i in p.indices(g.width) {
                    let mut changed=tiles.clone();changed[i].source.tile_index^=1;
                    assert!(!resolve(map,&changed.iter().collect::<Vec<_>>(),&g,None).iter().any(|q|q.kind==p.kind));
                }
                drop(refs);tiles[0].source.tileset_id="house".into();
                assert!(!resolve(map,&tiles.iter().collect::<Vec<_>>(),&g,None).iter().any(|q|matches!(q.kind,ModelKind::RetailRefrigerator|ModelKind::RetailShelf)));
            }
        }
    }

    #[test]
    fn kanto_native_retail_displays_keep_blue_glass_and_equipment_sources_without_generic_goods() {
        let mut blue=profile("Shared department store cushion 2");
        blue.name="Custom blue display".into();blue.metatile=0x12;blue.metatiles=None;
        blue.origin=[2,0];blue.ground=1;blue.map=None;blue.maps=None;blue.parts.clear();
        blue.tiles=vec![vec![0x26,0x27],vec![0x36,0x37],vec![0x28,0x29],vec![0x38,0x39]];
        let mut crossing=blue.clone();crossing.metatile=0x2c;crossing.origin=[2,2];
        crossing.metatiles=Some(vec![vec![0x2c],vec![0x2d]]);
        for (object,kind) in [(blue,ModelKind::RetailShelf),(crossing,ModelKind::RetailShelf),
            (profile("Shared Mart Ecruteak Mart refrigerator 0"),ModelKind::RetailRefrigerator),
            (profile("Shared Mart Ecruteak Mart shelf 0"),ModelKind::RetailShelf)] {
            for map in ["CeladonDeptStore2F","GoldenrodDeptStore1F","GoldenrodDeptStore2F",
                "GoldenrodDeptStore3F","GoldenrodDeptStore4F","GoldenrodDeptStore5F","GoldenrodDeptStore6F"] {
            let (mut tiles,g)=fixture(&object);
            // Department stores use tile 01 beneath these shared Mart fixtures;
            // the canonical ordinary-Mart profile supplies tile 48 instead.
            tiles.last_mut().unwrap().source.tile_index=crate::mart::FLOOR_TILE;
            let refs=tiles.iter().collect::<Vec<_>>();
            let placements=resolve(map,&refs,&g,Some(built_in_profiles()));
            let cabinet=placements.iter().find(|p|p.kind==kind)
                .unwrap_or_else(||panic!("complete native display missing: {}",object.name));
            let mut mesh=TerrainMeshData{footing_heights:vec![2.5;tiles.len()],..Default::default()};
            let mut claims=vec![false;tiles.len()];
            assert!(append(&mut mesh,map,&refs,&g,cabinet,&mut claims,[0,0]));
            assert!(mesh.solid.indices.is_empty(),"native source fronts replace generic merchandise paint");
            assert!(mesh.footing_heights.iter().all(|h|*h==2.5));
            assert_eq!(claims.iter().filter(|&&c|c).count(),8);
            let blue_case=tiles[cabinet.row*g.width+cabinet.column].source.tile_index==0x26;
            if blue_case {
                let scale=g.tile_height/8.;
                let vertices=&mesh.textured.positions;
                assert!((vertices.iter().map(|v|v[1]).fold(0.,f32::max)-8.6*scale).abs()<0.001);
                let min_z=vertices.iter().filter(|v|v[1]>0.001).map(|v|v[2]).fold(f32::INFINITY,f32::min);
                let max_z=vertices.iter().filter(|v|v[1]>0.001).map(|v|v[2]).fold(f32::NEG_INFINITY,f32::max);
                assert!((max_z-min_z-24.*scale).abs()<0.001,"long case follows its source plot");
                for i in mesh.textured.indices.iter().map(|&i|i as usize) {
                    let v=vertices[i];
                    if v[1]<8.*scale-0.001 && mesh.textured.normals[i][1].abs()<0.001 {
                        assert!(v[1].abs()<0.001,"sides stay flat from floor to lid; ridges never descend them");
                    }
                }
            } else {
            let top=cabinet.height_pixels*0.75*g.tile_height/8.;
            let (_,_,v0,_)=g.uv(cabinet.column,cabinet.row);
            let (_,_,_,v1)=g.uv(cabinet.column+1,cabinet.row+3);
            let door_v=v0+(v1-v0)*0.25;
            assert!(mesh.textured.positions.iter().all(|v|v[1]<=top+0.001),
                "roof perspective must not add another eight pixels to the cabinet front");
            for i in mesh.textured.indices.iter().map(|&i|i as usize) {
                if mesh.textured.normals[i][2]>0.5 {
                    assert!(mesh.textured.uvs[i][1]>=door_v-0.00001,
                        "vent header belongs on the roof, never on the door face");
                }
            }
            for x in 0..2 {
                let uv=g.uv(cabinet.column+x,cabinet.row);
                assert!((0..mesh.textured.positions.len()).any(|i|
                    (mesh.textured.positions[i][1]-top).abs()<0.001
                        &&mesh.textured.normals[i][1]>0.999
                        &&mesh.textured.uvs[i]==[uv.0,uv.2]),"source roof faces upward");
            }
            for y in 0..4 {for x in 0..2 {
                let uv=g.uv(x,y);
                assert!(mesh.textured.uvs.contains(&[uv.0,uv.2]));
                assert!(mesh.textured.uvs.contains(&[uv.1,uv.3]));
            }}
            let front=(cabinet.row+cabinet.height) as f32*g.tile_height;
            assert!(mesh.textured.positions.iter().any(|p|p[2]>front-1. && p[2]<front),"recessed front volume");
            }
            let mut johto=TerrainMeshData::default();
            assert!(append(&mut johto,"UnreviewedJohtoMart",&refs,&g,cabinet,&mut vec![false;tiles.len()],[0,0]));
            assert!(!johto.solid.indices.is_empty(),"unrelated model treatments stay unchanged");
            let mut changed=object.clone();changed.name="New customized retail object".into();
            changed.map=None;changed.maps=None;
            let custom=Document{objects:vec![changed],atmosphere:None};
            assert!(!resolve(map,&refs,&g,Some(&custom)).iter().any(|p|p.kind==kind));
            drop(refs);tiles[0].source.tile_index^=1;
            assert!(!resolve(map,&tiles.iter().collect::<Vec<_>>(),&g,None)
                .iter().any(|p|p.kind==kind));
            tiles[0].source.tile_index^=1;
            let mut clipped=g.clone();clipped.height=3;tiles.truncate(g.width*3);
            assert!(!resolve(map,&tiles.iter().collect::<Vec<_>>(),&clipped,None)
                .iter().any(|p|p.kind==kind));
            }
        }
    }

    #[test]
    fn kanto_native_cafe_table_uses_complete_source_and_closed_basket_without_fallback() {
        let mut object=profile("PlayersHouse1F dining table");
        object.name="Customized cafe table".into();object.tileset="game_corner".into();
        object.map=None;object.maps=None;object.metatiles=Some(vec![vec![0x2c],vec![0x2e]]);object.metatile=0x2e;
        object.origin=[0,3];object.ground=1;object.parts.clear();object.footing_pixels=None;
        object.tiles=vec![vec![0x2e,0x0d,0x0d,0x2f],vec![0x1c,0x20,0x21,0x1d],vec![0x1c,0x30,0x31,0x1d],
            vec![0x3e,0x50,0x50,0x3f],vec![0xa8,0x4e,0x4f,0xa9]];
        let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve("CeladonCafe",&refs,&g,Some(built_in_profiles()));
        let table=placements.iter().find(|p|p.kind==ModelKind::CafeTable).unwrap();
        assert_eq!(table.indices(g.width).count(),20);
        let mut mesh=TerrainMeshData{footing_heights:vec![0.75;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        assert!(append(&mut mesh,"CeladonCafe",&refs,&g,table,&mut claimed,[0,0]));
        assert_eq!(claimed.iter().filter(|&&c|c).count(),20);
        assert!(mesh.footing_heights.iter().all(|h|*h==0.75));
        assert!(mesh.solid.indices.is_empty());
        assert!(mesh.textured.positions.iter().any(|p|p[1]>18.));
        assert!(mesh.textured.positions.iter().all(|p|p[1]>=0. &&p[1]<=19.31));
        let mut basket=SurfaceMeshData::default();
        append_cafe_basket(&mut basket,[0.,0.],1.,[[0.1,0.2],[0.3,0.4],[0.5,0.6]]);
        for tri in basket.indices.chunks_exact(3) {
            let [a,b,c]=[0,1,2].map(|j|Vec3::from_array(basket.positions[tri[j] as usize]));
            assert!((b-a).cross(c-a).dot(Vec3::from_array(basket.normals[tri[0] as usize]))>0.);
        }
        let key=|p:[f32;3]|p.map(|v|(v*10000.).round()as i32);
        let mut edges=std::collections::HashMap::new();
        for tri in basket.indices.chunks_exact(3) {for (a,b) in [(0,1),(1,2),(2,0)] {
            let mut edge=[key(basket.positions[tri[a]as usize]),key(basket.positions[tri[b]as usize])];
            edge.sort();*edges.entry(edge).or_insert(0)+=1;
        }}
        assert!(edges.values().all(|count|*count==2),"basket and contents are closed");
        let custom=Document{objects:vec![object.clone()],atmosphere:None};
        assert!(!resolve("CeladonCafe",&refs,&g,Some(&custom)).iter().any(|p|p.kind==ModelKind::CafeTable));
        assert!(!resolve("GoldenrodGameCorner",&refs,&g,None).iter().any(|p|p.kind==ModelKind::CafeTable));
        drop(refs);tiles[0].source.tile_index^=1;
        assert!(!resolve("CeladonCafe",&tiles.iter().collect::<Vec<_>>(),&g,None)
            .iter().any(|p|p.kind==ModelKind::CafeTable));
        assert_eq!(crate::cafe::shape("CeladonCafe",&tiles[0].source),None);
    }

    #[test]
    fn kanto_native_gate_double_doors_keep_blue_faces_and_join_the_wall_course() {
        let object=profile("Shared gate north double doors");
        let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve("Route5SaffronGate",&refs,&g,Some(built_in_profiles()));
        let doors=placements.iter().find(|p|p.kind==ModelKind::GateDoorFrame).unwrap();
        let mut mesh=TerrainMeshData{footing_heights:vec![1.75;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        assert!(append(&mut mesh,"Route5SaffronGate",&refs,&g,doors,&mut claimed,[0,0]));
        assert!(mesh.solid.indices.is_empty(),"closed blue doors use their native atlas");
        assert!(mesh.footing_heights.iter().all(|h|*h==1.75));
        assert_eq!(claimed.iter().filter(|&&v|v).count(),12);
        assert!(mesh.textured.positions.iter().all(|p|p[1]>=0. && p[1]<=22.));
        assert!(mesh.textured.positions.iter().any(|p|p[1]==22. && p[2]==24.));
        let front=mesh.textured.indices.chunks_exact(3).filter(|t|
            mesh.textured.normals[t[0] as usize][2]>0.9).map(|t| {
                let p=[t[0],t[1],t[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                (p[1]-p[0]).cross(p[2]-p[0]).z*0.5
            }).sum::<f32>();
        assert!((front-32.*22.).abs()<0.01,"door panels close the entire wall course");
        for y in 0..3 {for x in 0..4 {
            let uv=g.uv(x,y);
            assert!(mesh.textured.uvs.contains(&[uv.1,uv.3]));
        }}
        for map in crate::mesh::gate_counters::JOHTO_GATE_ROOMS {
            let mut native=TerrainMeshData{footing_heights:vec![1.75;tiles.len()],..Default::default()};
            let mut claims=vec![false;tiles.len()];
            assert!(append(&mut native,map,&refs,&g,doors,&mut claims,[0,0]));
            assert!(native.solid.indices.is_empty());
            assert_eq!(native.footing_heights,vec![1.75;tiles.len()]);
            assert_eq!(claims.iter().filter(|&&v|v).count(),12);
        }
        let mut other=TerrainMeshData::default();
        assert!(append(&mut other,"UnreviewedJohtoGate",&refs,&g,doors,
            &mut vec![false;tiles.len()],[0,0]));
        assert!(!other.solid.indices.is_empty(),"unrelated gate treatment is preserved");
        let mut customized=object.clone();customized.name="Custom gate double doors".into();
        let custom=Document{objects:vec![customized],atmosphere:None};
        assert!(!resolve("Route5SaffronGate",&refs,&g,Some(&custom))
            .iter().any(|p|p.kind==ModelKind::GateDoorFrame));
        drop(refs);tiles[0].source.tile_index^=1;
        assert!(!resolve("Route5SaffronGate",&tiles.iter().collect::<Vec<_>>(),&g,None)
            .iter().any(|p|p.kind==ModelKind::GateDoorFrame));
    }

    #[test]
    fn kanto_native_gate_wall_fragments_keep_source_palette_and_authored_course_offsets() {
        for (name,base) in [("Shared gate wall variant 2d",0.),
            ("Shared gate counter junction upper wall",14.),
            ("Shared gate wall above planters",8.)] {
            let object=profile(name);let (mut tiles,g)=fixture(&object);
            let refs=tiles.iter().collect::<Vec<_>>();
            let placements=resolve("Route5SaffronGate",&refs,&g,Some(built_in_profiles()));
            let wall=placements.iter().find(|p|p.kind==ModelKind::WallClinical
                && p.column==0 && p.row==0 && p.width==object.tiles[0].len()
                && p.height==object.tiles.len()).unwrap();
            let mut mesh=TerrainMeshData{footing_heights:vec![1.75;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            assert!(append(&mut mesh,"Route5SaffronGate",&refs,&g,wall,&mut claimed,[0,0]));
            assert!(mesh.solid.indices.is_empty());
            assert!(mesh.footing_heights.iter().all(|h|*h==1.75));
            assert_eq!(claimed.iter().filter(|&&v|v).count(),wall.width*wall.height);
            let vertices=mesh.textured.indices.chunks_exact(3).filter(|t| {
                mesh.textured.normals[t[0] as usize][1]<0.9
            }).flatten().map(|i|mesh.textured.positions[*i as usize]).collect::<Vec<_>>();
            assert!(vertices.iter().all(|p|p[1]>=base && p[1]<=22.));
            assert!(vertices.iter().any(|p|p[1]==22.));
            assert!(vertices.iter().any(|p|p[2]==24.),"all fronts meet the same native course");
            for map in crate::mesh::gate_counters::JOHTO_GATE_ROOMS {
                let mut native=TerrainMeshData{footing_heights:vec![1.75;tiles.len()],..Default::default()};
                let mut claims=vec![false;tiles.len()];
                assert!(append(&mut native,map,&refs,&g,wall,&mut claims,[0,0]));
                assert!(native.solid.indices.is_empty());
                assert_eq!(native.footing_heights,vec![1.75;tiles.len()]);
                assert_eq!(claims.iter().filter(|&&v|v).count(),wall.width*wall.height);
            }
            let mut johto=TerrainMeshData::default();
            assert!(append(&mut johto,"UnreviewedJohtoGate",&refs,&g,wall,&mut vec![false;tiles.len()],[0,0]));
            assert!(!johto.solid.indices.is_empty());
            drop(refs);tiles[0].source.tile_index^=1;
            assert!(!resolve("Route5SaffronGate",&tiles.iter().collect::<Vec<_>>(),&g,None)
                .iter().any(|p|p.kind==ModelKind::WallClinical && p.column==0 &&p.row==0
                    && p.width==object.tiles[0].len() && p.height==object.tiles.len()));
        }
    }

    #[test]
    fn kanto_native_center_lounge_seat_keeps_low_cushion_palette_and_source_ownership() {
        let object=profile("Shared Pokecenter seat 2e");
        let (mut tiles,g)=fixture(&object);
        let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve("PewterPokecenter1F",&refs,&g,None);
        let seat=placements.iter().find(|p|p.kind==ModelKind::LinkSeat).unwrap();
        let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
        let mut claimed=vec![false;tiles.len()];
        assert!(append(&mut mesh,"PewterPokecenter1F",&refs,&g,seat,&mut claimed,[0,0]));
        assert!(mesh.solid.indices.is_empty(),"all lounge surfaces use its own source palette");
        assert_eq!(claimed.iter().filter(|&&v|v).count(),4);
        assert!(mesh.footing_heights.iter().all(|h|*h==1.25));
        assert!(mesh.textured.positions.iter().all(|p|p[1]>=0. && p[1]<=7.));
        assert!(mesh.textured.positions.iter().any(|p|p[1]==7.));
        assert!(mesh.textured.normals.iter().any(|n|n[1]==-1.),"closed cushion and feet");
        let mut johto=TerrainMeshData::default();
        assert!(append(&mut johto,"UnreviewedPokecenter1F",&refs,&g,seat,&mut vec![false;tiles.len()],[0,0]));
        assert!(!johto.solid.indices.is_empty(),"other rooms retain their existing treatment");
        drop(refs);
        tiles[0].source.tile_index^=1;
        assert!(!resolve("PewterPokecenter1F",&tiles.iter().collect::<Vec<_>>(),&g,None)
            .iter().any(|p|p.kind==ModelKind::LinkSeat),"changed cushion art is not replaced");
    }

    #[test]
    fn kanto_oak_workstation_keeps_native_desk_gray_console_and_exact_geometry() {
        let object=profile("Shared lab computer workstation");let (mut tiles,g)=fixture(&object);
        let refs=tiles.iter().collect::<Vec<_>>();
        let p=resolve("OaksLab",&refs,&g,Some(built_in_profiles())).into_iter()
            .find(|p|p.kind==ModelKind::LabWorkstation).unwrap();
        let (swatches,desk)=native_lab_workstation_source("OaksLab",&p,&refs,&g).unwrap();
        let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
        let mut claims=vec![false;tiles.len()];assert!(append(&mut mesh,"OaksLab",&refs,&g,&p,&mut claims,[0,0]));
        assert_eq!(claims.iter().filter(|&&v|v).count(),12);
        assert!(mesh.footing_heights.iter().all(|&h|h==1.25));assert!(mesh.solid.positions.is_empty());
        let mut other=TerrainMeshData::default();
        assert!(append(&mut other,"UnreviewedLab",&refs,&g,&p,&mut vec![false;tiles.len()],[0,0]));
        let floor_indices=other.textured.indices.len();
        let source_geometry=|surface:&SurfaceMeshData,skip:usize|surface.indices[skip..].iter()
            .map(|&i|(surface.positions[i as usize],surface.normals[i as usize])).collect::<Vec<_>>();
        assert_eq!(source_geometry(&mesh.textured,floor_indices),source_geometry(&other.solid,0),
            "native paint retains every fitted triangle position and normal");
        assert!(mesh.textured.uvs.contains(&desk[0]),"source gold desk region retained");
        assert!(mesh.textured.uvs.contains(&swatches[2]),"source gray console retained");
        assert!(mesh.textured.uvs.contains(&swatches[6]),"source dark display retained");
        for (&position,&uv) in mesh.textured.positions.iter().zip(&mesh.textured.uvs) {
            if desk.contains(&uv) {
                assert!((9.6..=11.7).contains(&position[1]),
                    "gold stays on the fitted worktop, away from tower and keyboard");
            }
        }
        for tri in mesh.textured.indices[floor_indices..].chunks_exact(3) {
            assert!(tri.iter().all(|&i|mesh.textured.uvs[i as usize]==mesh.textured.uvs[tri[0]as usize]),
                "whole painted face must not interpolate across atlas regions");
        }
        drop(refs);
        for i in p.indices(g.width) {tiles[i].source.tile_index^=1;
            assert!(!resolve("OaksLab",&tiles.iter().collect::<Vec<_>>(),&g,None).iter()
                .any(|p|p.kind==ModelKind::LabWorkstation));tiles[i].source.tile_index^=1;}
        let mut changed=object;changed.name="Custom clinical workstation".into();
        let custom=Document{objects:vec![changed],atmosphere:None};
        assert!(!resolve("OaksLab",&tiles.iter().collect::<Vec<_>>(),&g,Some(&custom)).iter()
            .any(|p|p.kind==ModelKind::LabWorkstation));
    }
    #[test]
    fn kanto_moon_gift_fixtures_keep_native_art_scale_claims_and_footing() {
        for (kind,block,origin,art) in [
            (ModelKind::GiftShelf,2,[0,0],vec![vec![6,7,7,32],vec![22,23,23,48],vec![33,39,39,40],vec![49,55,55,56]]),
            (ModelKind::LabCounter,18,[0,0],vec![vec![35,36],vec![66,67],vec![66,67],vec![51,52]]),
            (ModelKind::WallTraditional,8,[0,0],vec![vec![17;4],vec![17;4],vec![80;4],vec![80;4]]),
            (ModelKind::RadioWall,1,[0,0],vec![vec![17;4],vec![17;4],vec![10,11,80,80],vec![26,27,80,80]]),
            (ModelKind::PendulumClock,19,[0,0],vec![vec![35,36],vec![41,42],vec![57,58],vec![82,83]]),
            (ModelKind::Cabinet,19,[2,1],vec![vec![35,36],vec![62,63],vec![24,25]]),
            (ModelKind::Cushion,16,[0,0],vec![vec![68,69,2,3],vec![84,85,18,19],vec![69,70,68,69],vec![85,86,84,85]]),
        ] {
            let mut object=profile(if kind==ModelKind::Cushion {"Traditional cushion 10"}else{"Traditional house north clock"});object.metatile=block;object.origin=origin;object.tiles=art;
            if kind==ModelKind::GiftShelf {object.ground=1;}
            let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let all=resolve("MountMoonGiftShop",&refs,&g,None);
            let p=all.iter().find(|p|p.kind==kind).unwrap_or_else(||panic!("missing {kind:?}: {all:?}")).clone();
            assert_eq!(native_gift_shop::source("MountMoonGiftShop",&p,&refs,&g),Some(kind));
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
            let mut claims=vec![false;tiles.len()];assert!(append(&mut mesh,"MountMoonGiftShop",&refs,&g,&p,&mut claims,[0,0]));
            assert_eq!(claims.iter().filter(|&&v|v).count(),p.width*p.height);
            assert!(mesh.solid.positions.is_empty());assert!(mesh.footing_heights.iter().all(|&h|h==1.25));
            if kind==ModelKind::GiftShelf||kind==ModelKind::LabCounter {
                let expected=if kind==ModelKind::GiftShelf {16.}else{8.};
                let body=&mesh.textured.positions[p.width*p.height*4..];
                assert_eq!(body.iter().map(|p|p[1]).fold(0.,f32::max),expected);
                let mut area=0.;
                for tri in mesh.textured.indices[p.width*p.height*6..].chunks_exact(3) {
                    let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                    let n=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);
                    let cross=(points[1]-points[0]).cross(points[2]-points[0]);assert!(cross.dot(n)>0.);
                    if n==Vec3::Z {area+=cross.length()*0.5;}
                }
                let inset=if kind==ModelKind::GiftShelf {0.72}else{0.};
                assert!((area-(p.width as f32*8.-inset)*expected).abs()<0.001,"single complete front");
            }
            assert_eq!(native_gift_shop::source("MahoganyMart1F",&p,&refs,&g),
                matches!(kind,ModelKind::GiftShelf|ModelKind::PendulumClock|ModelKind::Cabinet
                    |ModelKind::DrawerCabinet|ModelKind::WallTraditional|ModelKind::RadioWall).then_some(kind));
            if kind==ModelKind::GiftShelf {
                let mut shop=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
                let mut shop_claims=vec![false;tiles.len()];
                assert!(append(&mut shop,"MahoganyMart1F",&refs,&g,&p,&mut shop_claims,[0,0]));
                assert!(shop.solid.positions.is_empty());
                assert_eq!(shop_claims.iter().filter(|&&v|v).count(),16);
                assert_eq!(shop.footing_heights,vec![1.25;tiles.len()]);
                assert_eq!(shop.textured.positions.iter().map(|p|p[1]).fold(0.,f32::max),16.);
                assert!(native_gift_shop::source("MahoganyMart1FAnnex",&p,&refs,&g).is_none());
            }
            drop(refs);
            for index in p.indices(g.width) {tiles[index].source.tile_index^=1;
                assert!(native_gift_shop::source("MountMoonGiftShop",&p,&tiles.iter().collect::<Vec<_>>(),&g).is_none());
                assert!(native_gift_shop::source("MahoganyMart1F",&p,&tiles.iter().collect::<Vec<_>>(),&g).is_none());
                tiles[index].source.tile_index^=1;}
        }
    }
    #[test]
    fn johto_traditional_cushions_keep_native_blue_art_and_authoritative_footing() {
        for map in ["DanceTheater","KurtsHouse","EcruteakItemfinderHouse",
            "EcruteakLugiaSpeechHouse","MahoganyMart1F","VioletKylesHouse",
            "VioletNicknameSpeechHouse"] {
            for name in ["Traditional cushion 10","Traditional cushion 1b","Traditional cushion 19"] {
                let object=profile(name);
                let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
                let p=resolve(map,&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::Cushion).unwrap();
                assert!(native_gift_shop::traditional_cushion(map,&p,&refs,&g));
                let raised=Placement{base_pixels:7.0,..p.clone()};
                assert!(!native_gift_shop::traditional_cushion(map,&raised,&refs,&g),
                    "custom vertical placement retains its own rendering path");
                assert!(!native_gift_shop::traditional_cushion("UnreviewedTraditionalHouse",&p,&refs,&g));
                let mut mesh=TerrainMeshData{footing_heights:vec![3.25;tiles.len()],..Default::default()};
                let mut claims=vec![false;tiles.len()];
                assert!(append(&mut mesh,map,&refs,&g,&p,&mut claims,[0,0]));
                assert_eq!(claims.iter().filter(|&&v|v).count(),4);
                assert!(mesh.solid.positions.is_empty());
                assert!(mesh.footing_heights.iter().all(|&h|h==3.25));
                let top=p.height_pixels*g.tile_height/8.+0.002*g.tile_height/8.;
                assert!(mesh.textured.positions.iter().any(|v|(v[1]-top).abs()<0.00001));
                drop(refs);
                for i in p.indices(g.width) {
                    tiles[i].source.tile_index^=1;
                    assert!(!native_gift_shop::traditional_cushion(map,&p,&tiles.iter().collect::<Vec<_>>(),&g));
                    tiles[i].source.tile_index^=1;
                }
            }
        }
    }
    #[test]
    fn johto_traditional_clocks_and_cabinets_keep_native_wood_and_source_ownership() {
        for map in ["DanceTheater","KurtsHouse","EcruteakItemfinderHouse",
            "EcruteakLugiaSpeechHouse","MahoganyMart1F","VioletKylesHouse",
            "VioletNicknameSpeechHouse"] {
            for (name,kind) in [("Traditional house north clock",ModelKind::PendulumClock),
                ("Traditional house north cabinet",ModelKind::Cabinet),
                ("Traditional house low cabinet",ModelKind::Cabinet),
                ("Traditional house drawer cabinet",ModelKind::DrawerCabinet)] {
                let object=profile(name);let (tiles,g)=fixture(&object);
                let refs=tiles.iter().collect::<Vec<_>>();
                let p=resolve(map,&refs,&g,None).into_iter().find(|p|p.kind==kind).unwrap();
                assert_eq!(native_gift_shop::source(map,&p,&refs,&g),Some(kind));
                assert!(native_gift_shop::source("UnreviewedTraditionalHouse",&p,&refs,&g).is_none());
                let raised=Placement{base_pixels:4.,..p.clone()};
                assert!(native_gift_shop::source(map,&raised,&refs,&g).is_none());
                let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
                let mut claims=vec![false;tiles.len()];assert!(append(&mut mesh,map,&refs,&g,&p,&mut claims,[0,0]));
                assert!(mesh.solid.positions.is_empty());
                assert_eq!(claims.iter().filter(|&&v|v).count(),p.width*p.height);
                assert_eq!(mesh.footing_heights,vec![1.25;tiles.len()]);
                assert!((mesh.textured.positions.iter().map(|p|p[1]).fold(0.,f32::max)-p.height_pixels).abs()<1e-4);
                for i in p.indices(g.width) {for mode in 0..3 {
                    let mut changed=tiles.clone();let s=&mut changed[i].source;
                    match mode {0=>s.tile_index^=1,1=>s.metatile_id^=1,_=>s.subtile_column^=1};
                    assert!(native_gift_shop::source(map,&p,&changed.iter().collect::<Vec<_>>(),&g).is_none());
                }}
            }
        }
    }
    #[test]
    fn johto_traditional_tables_keep_gold_source_paint_height_and_claims() {
        let mut object=profile("Traditional cushion 10");
        object.origin=[2,2];object.ground=0x50;object.parts.clear();
        object.metatiles=Some(vec![vec![0x1b,0x1c],vec![0x18,0x19]]);
        object.tiles=vec![vec![0x23,0x22,0x22,0x24],vec![0x42,0x15,0x15,0x43],
            vec![0x42,0x15,0x15,0x43],vec![0x33,0x32,0x32,0x34]];
        for map in ["DanceTheater","KurtsHouse","EcruteakItemfinderHouse",
            "EcruteakLugiaSpeechHouse","MahoganyMart1F","VioletKylesHouse",
            "VioletNicknameSpeechHouse"] {
            let (tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let p=resolve(map,&refs,&g,None).into_iter().find(|p|p.kind==ModelKind::LowTable).unwrap();
            assert!(native_gift_shop::traditional_table(map,&p,&refs,&g));
            for y in 0..g.height {for x in 0..g.width {
                let expected=(x>=p.column&&x<p.column+4&&y>=p.row&&y<p.row+4).then_some(4.);
                assert_eq!(traditional_tabletop_height(map,&refs,&g,None,x,y),expected);
            }}
            let mut scaled=g.clone();scaled.tile_height*=2.;scaled.tile_width*=2.;
            assert_eq!(traditional_tabletop_height(map,&refs,&scaled,None,p.column,p.row),Some(8.));
            let mut custom=object.clone();custom.name="Custom traditional table".into();
            let active=Document{objects:vec![custom],atmosphere:None};
            assert_eq!(traditional_tabletop_height(map,&refs,&g,Some(&active),p.column,p.row),None);
            assert!(!native_gift_shop::traditional_table("UnreviewedTraditionalHouse",&p,&refs,&g));
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
            let mut claims=vec![false;tiles.len()];assert!(append(&mut mesh,map,&refs,&g,&p,&mut claims,[0,0]));
            assert!(mesh.solid.positions.is_empty());
            assert_eq!(claims.iter().filter(|&&v|v).count(),16);
            assert_eq!(mesh.footing_heights,vec![1.25;tiles.len()]);
            assert_eq!(mesh.textured.positions.iter().map(|p|p[1]).fold(0.,f32::max),4.);
            let swatches=native_source_swatches(&p,&g,[[0,24],[9,8],[1,24]]);
            assert!(swatches.iter().all(|uv|mesh.textured.uvs.contains(uv)));
            for i in p.indices(g.width) {for mode in 0..3 {
                let mut changed=tiles.clone();let s=&mut changed[i].source;
                match mode {0=>s.tile_index^=1,1=>s.metatile_id^=1,_=>s.subtile_column^=1};
                assert!(!native_gift_shop::traditional_table(map,&p,&changed.iter().collect::<Vec<_>>(),&g));
                assert_eq!(traditional_tabletop_height(map,&changed.iter().collect::<Vec<_>>(),&g,
                    None,p.column,p.row),None);
            }}
        }
    }
    #[test]
    fn kanto_clinic_mirrored_counter_wall_requires_full_block_and_leaves_lower_half() {
        let mut object=profile("Shared Pokecenter healing machine");
        object.metatile=42;object.tiles=vec![vec![2,2,2,2],vec![15,2,2,2],vec![15,1,1,1],vec![15,17,17,17]];
        let (mut tiles,g)=fixture(&object);
        let check=|tiles:&[VisualTile]|resolve("IndigoPlateauPokecenter1F",&tiles.iter().collect::<Vec<_>>(),&g,None)
            .into_iter().filter(|p|p.kind==ModelKind::WallClinical&&p.row==0).collect::<Vec<_>>();
        let placements=check(&tiles);assert_eq!(placements.len(),2);
        assert_eq!(placements.iter().flat_map(|p|p.indices(g.width)).count(),8);
        assert!(placements.iter().flat_map(|p|p.indices(g.width)).all(|i|i/g.width<2));
        for y in 0..4 {for x in 0..4 {let i=y*g.width+x;tiles[i].source.tile_index^=1;
            assert!(check(&tiles).is_empty(),"the whole original block guards the wall");tiles[i].source.tile_index^=1;}}
        let refs=tiles.iter().collect::<Vec<_>>();let mut claimed=vec![false;tiles.len()];claimed[2*g.width]=true;
        let mut out=Vec::new();native_clinic::append_mirrored_wall("IndigoPlateauPokecenter1F",&refs,&g,&mut claimed,&mut out);
        assert!(out.is_empty(),"custom lower ownership protects the whole drawing");
    }
    #[test]
    fn kanto_clinic_pc_header_joins_wall_without_claiming_computer() {
        let object=profile("Shared Pokecenter complete PC");let (mut tiles,mut g)=fixture(&object);
        let mut header=tiles[tiles.len()-g.width..].to_vec();
        for (x,tile) in header.iter_mut().take(2).enumerate() {
            tile.source.metatile_id=8;tile.source.subtile_column=2+x as u8;
            tile.source.subtile_row=0;tile.source.tile_index=2;
        }
        header.append(&mut tiles);tiles=header;g.height+=1;
        for (i,tile) in tiles.iter_mut().enumerate(){tile.column=(i%g.width)as u32;tile.row=(i/g.width)as u32;}
        let refs=tiles.iter().collect::<Vec<_>>();let placements=resolve("PewterPokecenter1F",&refs,&g,None);
        let p=placements.iter().find(|p|p.kind==ModelKind::WallClinical&&p.height==1).unwrap();
        assert!(native_clinic::wall_source("PewterPokecenter1F",p,&refs,&g));
        assert_eq!((p.row,p.height,p.base_pixels,p.height_pixels),(0,1,16.,16.));
        let pc=placements.iter().find(|p|p.kind==ModelKind::Computer).unwrap();
        assert!(p.indices(g.width).all(|i|!pc.indices(g.width).any(|j|j==i)));
        let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
        let mut claims=vec![false;tiles.len()];assert!(append(&mut mesh,"PewterPokecenter1F",&refs,&g,p,&mut claims,[0,0]));
        assert_eq!(claims.iter().filter(|&&v|v).count(),2);
        assert!(mesh.footing_heights.iter().all(|&h|h==1.25));
        let body=&mesh.textured.positions[2*4..];
        assert_eq!(body.iter().map(|p|p[1]).fold(f32::INFINITY,f32::min),16.);
        assert_eq!(body.iter().map(|p|p[1]).fold(0.,f32::max),32.);
        assert_eq!(body.iter().map(|p|p[2]).fold(0.,f32::max),16.);
        drop(refs);
        tiles[0].source.tile_index=3;
        assert!(!resolve("PewterPokecenter1F",&tiles.iter().collect::<Vec<_>>(),&g,None).iter()
            .any(|p|p.kind==ModelKind::WallClinical&&p.height==1));
        tiles[0].source.tile_index=2;
        let mut custom=object;custom.depth_pixels+=1.;
        assert!(!resolve("PewterPokecenter1F",&tiles.iter().collect::<Vec<_>>(),&g,
            Some(&Document{objects:vec![custom],atmosphere:None})).iter()
            .any(|p|p.kind==ModelKind::WallClinical&&p.height==1));
    }
    #[test]
    fn kanto_clinic_walls_keep_complete_native_courses_and_closed_bounds() {
        for (block,column,art) in [(2,2,vec![vec![2,2],vec![2,2]]),
            (3,0,vec![vec![2,2],vec![2,2]]),(3,2,vec![vec![2,2],vec![2,15]]),
            (8,0,vec![vec![2,2],vec![2,2]]),(19,0,vec![vec![2,2],vec![2,2]]),
            (19,2,vec![vec![2,2],vec![2,2]])] {
            let mut object=profile("Shared Pokecenter complete PC");
            object.metatile=block;object.origin=[column,0];object.tiles=art;
            let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
            let p=resolve("PewterPokecenter1F",&refs,&g,None).into_iter()
                .find(|p|native_clinic::wall_source("PewterPokecenter1F",p,&refs,&g)).unwrap();
            let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
            let mut claims=vec![false;tiles.len()];assert!(append(&mut mesh,"PewterPokecenter1F",&refs,&g,&p,&mut claims,[0,0]));
            assert_eq!(claims.iter().filter(|&&v|v).count(),4);assert!(mesh.solid.positions.is_empty());
            assert!(mesh.footing_heights.iter().all(|&h|h==1.25));
            let mut area=0.;
            for tri in mesh.textured.indices[4*6..].chunks_exact(3) {
                let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                let normal=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);
                let cross=(points[1]-points[0]).cross(points[2]-points[0]);
                assert!(cross.dot(normal)>0.);
                if normal==Vec3::Z {area+=cross.length()*0.5;}
            }
            assert!((area-16.*32.).abs()<0.001,"exactly one complete ceiling-height front, no layered face");
            for y in 0..2 {for x in 0..2 {let uv=g.uv(p.column+x,p.row+y);
                assert!(mesh.textured.uvs[4*4..].contains(&[uv.0,uv.2]));}}
            assert!(!native_clinic::wall_source("UnreviewedPokecenter1F",&p,&refs,&g));
            drop(refs);
            for i in p.indices(g.width) {tiles[i].source.tile_index^=1;
                assert!(!native_clinic::wall_source("PewterPokecenter1F",&p,&tiles.iter().collect::<Vec<_>>(),&g));
                tiles[i].source.tile_index^=1;}
        }
    }
    #[test]
    fn kanto_clinic_blue_cabinet_aligns_above_console_and_preserves_source_guard() {
        let mut object=profile("Shared Pokecenter complete PC");
        object.metatile=2;object.origin=[0,0];object.tiles=vec![vec![4,5],vec![20,21]];
        let (mut tiles,g)=fixture(&object);let refs=tiles.iter().collect::<Vec<_>>();
        let p=resolve("PewterPokecenter1F",&refs,&g,None).into_iter()
            .find(|p|native_clinic::cabinet_source("PewterPokecenter1F",p,&refs,&g)).unwrap();
        let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
        let mut claims=vec![false;tiles.len()];assert!(append(&mut mesh,"PewterPokecenter1F",&refs,&g,&p,&mut claims,[0,0]));
        assert_eq!(claims.iter().filter(|&&v|v).count(),4);assert!(mesh.solid.positions.is_empty());
        assert!(mesh.footing_heights.iter().all(|&h|h==1.25));
        let body=&mesh.textured.positions[4*4..];
        assert_eq!(body.iter().map(|p|p[1]).fold(f32::INFINITY,f32::min),13.);
        assert_eq!(body.iter().map(|p|p[1]).fold(0.,f32::max),29.);
        assert_eq!(body.iter().map(|p|p[2]).fold(0.,f32::max),32.);
        for y in 0..2 {for x in 0..2 {let uv=g.uv(p.column+x,p.row+y);
            assert!(mesh.textured.uvs[4*4..].contains(&[uv.0,uv.2]));}}
        assert!(!native_clinic::cabinet_source("UnreviewedPokecenter1F",&p,&refs,&g));
        drop(refs);
        for i in p.indices(g.width) {tiles[i].source.tile_index^=1;
            assert!(!native_clinic::cabinet_source("PewterPokecenter1F",&p,&tiles.iter().collect::<Vec<_>>(),&g));
            tiles[i].source.tile_index^=1;}
    }
    #[test]
    fn johto_reviewed_clinics_use_native_equipment_and_seats_with_unchanged_claims() {
        for map in ["AzaleaPokecenter1F","BlackthornPokecenter1F","CherrygrovePokecenter1F","CianwoodPokecenter1F","EcruteakPokecenter1F","GoldenrodPokecenter1F","MahoganyPokecenter1F","OlivinePokecenter1F","Route32Pokecenter1F","SilverCavePokecenter1F","VioletPokecenter1F"] {
            for (name,kind,count) in [
                ("Shared Pokecenter complete PC",ModelKind::Computer,6),
                ("Shared Pokecenter healing machine",ModelKind::HealingMachine,16),
                ("Shared Pokecenter seat 2e",ModelKind::LinkSeat,4),
            ] {
                let object=profile(name);let (mut tiles,g)=fixture(&object);
                let refs=tiles.iter().collect::<Vec<_>>();
                let p=resolve(map,&refs,&g,Some(built_in_profiles())).into_iter()
                    .find(|p|p.kind==kind).unwrap();
                let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
                let mut claims=vec![false;tiles.len()];
                assert!(append(&mut mesh,map,&refs,&g,&p,&mut claims,[0,0]));
                assert!(mesh.solid.positions.is_empty(),"generic blue/cream paint cannot remain");
                assert_eq!(claims.iter().filter(|&&v|v).count(),count);
                assert!(mesh.footing_heights.iter().all(|&h|h==1.25));
                for i in p.indices(g.width) {
                    let old=tiles[i].source.tile_index;tiles[i].source.tile_index=0xffff;
                    assert!(!resolve(map,&tiles.iter().collect::<Vec<_>>(),&g,None).iter().any(|p|p.kind==kind));
                    tiles[i].source.tile_index=old;
                }
            }
        }
        assert!(!kanto_native_floor_map("UnreviewedPokecenter1F"));
    }

    #[test]
    fn kanto_clinic_pc_keeps_all_source_courses_and_footing() {
        let object=profile("Shared Pokecenter complete PC");let (mut tiles,g)=fixture(&object);
        let refs=tiles.iter().collect::<Vec<_>>();
        let p=resolve("PewterPokecenter1F",&refs,&g,Some(built_in_profiles())).into_iter()
            .find(|p|p.kind==ModelKind::Computer).unwrap();
        assert!(native_clinic::pc_source("PewterPokecenter1F",&p,&refs,&g));
        let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
        let mut claims=vec![false;tiles.len()];assert!(append(&mut mesh,"PewterPokecenter1F",&refs,&g,&p,&mut claims,[0,0]));
        assert_eq!(claims.iter().filter(|&&v|v).count(),6);assert!(mesh.solid.positions.is_empty());
        assert!(mesh.footing_heights.iter().all(|&h|h==1.25));
        let mut front_z=None;
        for tri in mesh.textured.indices[6*6..].chunks_exact(3) {
            let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
            let normal=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);
            assert!((points[1]-points[0]).cross(points[2]-points[0]).dot(normal)>0.);
            if normal==Vec3::Z {assert_eq!(*front_z.get_or_insert(points[0].z),points[0].z);}
        }
        for y in 0..3 {for x in 0..2 {let uv=g.uv(p.column+x,p.row+y);
            assert!(mesh.textured.uvs[6*4..].contains(&[uv.0,uv.2]));}}
        assert!(!native_clinic::pc_source("UnreviewedPokecenter1F",&p,&refs,&g));
        drop(refs);
        for i in p.indices(g.width) {tiles[i].source.tile_index^=1;
            assert!(!native_clinic::pc_source("PewterPokecenter1F",&p,&tiles.iter().collect::<Vec<_>>(),&g));
            tiles[i].source.tile_index^=1;}
    }
    #[test]
    fn kanto_native_healing_enclosure_has_empty_bay_and_complete_source_face() {
        let object=profile("Shared Pokecenter healing machine");let (mut tiles,g)=fixture(&object);
        let refs=tiles.iter().collect::<Vec<_>>();
        let p=resolve("PewterPokecenter1F",&refs,&g,Some(built_in_profiles())).into_iter()
            .find(|p|p.kind==ModelKind::HealingMachine).unwrap();
        assert!(native_clinic::healing_source("PewterPokecenter1F",&p,&refs,&g));
        let mut mesh=TerrainMeshData{footing_heights:vec![1.25;tiles.len()],..Default::default()};
        let mut claims=vec![false;tiles.len()];assert!(append(&mut mesh,"PewterPokecenter1F",&refs,&g,&p,&mut claims,[0,0]));
        assert_eq!(claims.iter().filter(|&&v|v).count(),16);assert!(mesh.solid.positions.is_empty());
        assert!(mesh.footing_heights.iter().all(|&h|h==1.25));
        let floor_indices=16*6;let mut front_area=0.;let mut bay_z=None;let mut frame_z=None;
        for tri in mesh.textured.indices[floor_indices..].chunks_exact(3) {
            let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
            let n=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);
            let area=(points[1]-points[0]).cross(points[2]-points[0]);
            assert!(area.dot(n)>0.,"all enclosure/bay faces point outward");
            if n==Vec3::Z {
                front_area+=area.length()*0.5;
                let y=points.iter().map(|p|p.y).sum::<f32>()/3.;
                let x=points.iter().map(|p|p.x).sum::<f32>()/3.;
                if x>8.&&x<24.&&y>7.5&&y<22.5 {bay_z=Some(points[0].z);}else{frame_z=Some(points[0].z);}
            }
        }
        assert!(bay_z.unwrap()<frame_z.unwrap(),"idle bay is recessed and has no raised capsules");
        let bounds=mesh.textured.positions[16*4..].iter().fold([f32::INFINITY,f32::NEG_INFINITY],|[lo,hi],p|[lo.min(p[0]),hi.max(p[0])]);
        assert!((front_area-(bounds[1]-bounds[0]-1.)*30.).abs()<0.001,"complete native front is retained");
        for y in 0..4 {for x in 0..4 {let uv=g.uv(p.column+x,p.row+y);
            assert!(mesh.textured.uvs[16*4..].contains(&[uv.0,uv.2]),"each source course retained");}}
        assert!(!native_clinic::healing_source("UnreviewedPokecenter1F",&p,&refs,&g));
        drop(refs);
        for i in p.indices(g.width) {tiles[i].source.tile_index^=1;
            assert!(!resolve("PewterPokecenter1F",&tiles.iter().collect::<Vec<_>>(),&g,None).iter()
                .any(|p|p.kind==ModelKind::HealingMachine));tiles[i].source.tile_index^=1;}
    }
    #[test]
    fn complete_machine_claims_only_its_drawing_and_appends_original_ground() {
        let object = profile("Shared Pokecenter healing machine");
        let (tiles, g) = fixture(&object);
        let refs: Vec<_> = tiles.iter().collect();
        let placements = resolve_fixture(&tiles, &g, None);
        let machine = placements
            .iter()
            .find(|p| p.kind == ModelKind::HealingMachine)
            .unwrap();
        assert_eq!(machine.indices(g.width).count(), 16);
        let mut claimed = vec![false; tiles.len()];
        let mut mesh = TerrainMeshData::default();
        mesh.authored_cells = vec![None; tiles.len()];
        mesh.footing_heights = vec![0.0; tiles.len()];
        assert!(append(&mut mesh, "UnreviewedPokecenter1F", &refs, &g, machine, &mut claimed, [0, 0]));
        assert_eq!(claimed.iter().filter(|&&v| v).count(), 16);
        assert_eq!(
            mesh.authored_cells.iter().filter(|v| v.is_some()).count(),
            16
        );
        assert!(mesh.solid.positions.len() > 100);
        assert_eq!(mesh.textured.quad_count(), 16);
        let size = mesh.solid.positions.len();
        assert!(!append(&mut mesh, "UnreviewedPokecenter1F", &refs, &g, machine, &mut claimed, [0, 0]));
        assert_eq!(mesh.solid.positions.len(), size);
        assert!(mesh.footing_heights.iter().all(|&v| v == 0.0));
    }
    #[test]
    fn department_fourth_floor_fixtures_use_their_actual_floor_sample() {
        for name in ["Shared Mart Ecruteak Mart refrigerator 0",
            "Shared Mart Ecruteak Mart refrigerator 2",
            "Shared Mart Ecruteak Mart shelf 0", "Shared Mart Ecruteak Mart shelf 2"] {
            let object = profile(name);
            let expected = profile_kind(&object).unwrap();
            let (mut tiles, geometry) = fixture(&object);
            for tile in &mut tiles {
                if tile.source.metatile_id == 0 {
                    tile.source.tile_index = 0x01;
                }
            }
            let cells = tiles.iter().collect::<Vec<_>>();
            for map in ["CeladonDeptStore4F", "GoldenrodDeptStore4F"] {
                let placements = resolve(map, &cells, &geometry, None);
                let placement = placements.iter().find(|p| p.kind == expected).unwrap();
                assert_eq!(cells[placement.ground].source.tile_index, 0x01);
            }
        }
    }
    #[test]
    fn changed_phase_wrong_art_and_missing_ground_are_unclaimed() {
        let object = profile("Shared Pokecenter healing machine");
        let (tiles, g) = fixture(&object);
        let mut wrong = tiles.clone();
        wrong[0].source.subtile_column = 1;
        assert!(
            resolve_fixture(&wrong, &g, None)
                .iter()
                .all(|p| p.kind != ModelKind::HealingMachine)
        );
        let mut wrong = tiles.clone();
        wrong[0].source.tile_index = u16::MAX;
        assert!(
            resolve_fixture(&wrong, &g, None)
                .iter()
                .all(|p| p.kind != ModelKind::HealingMachine)
        );
        let mut missing = tiles.clone();
        for tile in &mut missing {
            if tile.source.tile_index == object.ground {
                tile.source.tile_index = u16::MAX;
            }
        }
        assert!(resolve_fixture(&missing, &g, None).is_empty());
        let (_, mut clipped) = fixture(&object);
        clipped.width = 3;
        clipped.height = 3;
        let clipped_tiles: Vec<_> = tiles.iter().take(9).cloned().collect();
        assert!(
            resolve_fixture(&clipped_tiles, &clipped, None)
                .iter()
                .all(|p| p.kind != ModelKind::HealingMachine)
        );
    }
    #[test]
    fn changed_live_profile_keeps_user_geometry_ownership() {
        let mut object = profile("Shared Pokecenter complete PC");
        let (tiles, g) = fixture(&object);
        object.depth_pixels += 1.0;
        let custom = Document {
            objects: vec![object],
            atmosphere: None,
        };
        assert!(
            resolve_fixture(&tiles, &g, Some(&custom))
                .iter()
                .all(|p| p.kind != ModelKind::Computer)
        );
    }
    #[test]
    fn all_bound_profiles_have_specific_materialed_asset_roles() {
        let profiles: Vec<_> = built_in_profiles()
            .objects
            .iter()
            .filter(|p| profile_kind(p).is_some())
            .collect();
        assert!(profiles.len() >= 80);
        for p in profiles {
            let kind = profile_kind(p).unwrap();
            assert!(kind.label().starts_with("interior/"));
            let (depth, rise) = proportions(kind, p.tiles[0].len(), p.tiles.len());
            assert!(depth > 0.0 && depth <= p.tiles.len() as f32 * 8.0);
            assert!(rise > 0.0 && rise.is_finite());
        }
    }
}

#[cfg(test)]
#[path = "cable_fixture_tests.rs"]
mod cable_fixture_tests;

include!("interior_signatures.rs");

include!("interior_surfaces.rs");
include!("enhanced_room_floor.rs");
include!("casino_interiors.rs");
include!("cianwood_walls.rs");

include!("lighthouse_chamber_floor.rs");

include!("bedroom_carpet.rs");

include!("interior_variation.rs");

include!("prize_booth.rs");
