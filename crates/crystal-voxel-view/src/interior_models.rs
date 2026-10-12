//! Original, independently authored domestic and institutional furnishing kit.
//! Only geometry and materials are embedded. Native drawing identities live in
//! the placement module; collision and runtime content are never modified.
use crate::mesh::SurfaceMeshData;
use bevy::prelude::Vec3;
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ModelKind {
    Stool,
    ArcadeStool,
    TrashCan,
    Chair,
    LinkSeat,
    Cushion,
    DiningTable,
    LowTable,
    CafeTable,
    BedRed,
    BedBlue,
    BedGreen,
    BedPink,
    PottedPlant,
    LongPlanter,
    FlowerStand,
    Bookcase,
    LabBookcase,
    Cabinet,
    DrawerCabinet,
    PendulumClock,
    Computer,
    LabWorkstation,
    Television,
    Radio,
    GameConsole,
    Keyboard,
    Refrigerator,
    RetailRefrigerator,
    KitchenStove,
    KitchenSink,
    RetailShelf,
    GiftShelf,
    ReceptionCounter,
    LabCounter,
    HealingMachine,
    HealingConsole,
    ClinicalPartition,
    TimberPartition,
    LabRestorationMachine,
    LabDisplayTable,
    ArcadeMachine,
    ArcadePair,
    PrizeBooth,
    ArcadeWall,
    ArcadePrizeRack,
    OlivineStage,
    StationBench,
    StationTurnstile,
    GateTerminal,
    BroadcastConsole,
    VendingMachine,
    WallPanel,
    WindowWall,
    GateDoorFrame,
    RadioWall,
    OpenBook,
    Memorial,
    BedFeathery,
    BedPolkadot,
    BedPikachu,
    PlantMagna,
    PlantTropic,
    PlantJumbo,
    WallDomestic,
    WallTraditional,
    WallClinical,
    WallAcoustic,
    PictureFrame,
    CarpetCloth,
    StairFlight,
    OfficePhone,
    BroadcastRack,
    TowerReception,
}
impl ModelKind {
    pub(crate) const ALL: &'static [Self] = &[
        Self::Stool,
        Self::ArcadeStool,
        Self::TrashCan,
        Self::Chair,
        Self::LinkSeat,
        Self::Cushion,
        Self::DiningTable,
        Self::LowTable,
        Self::CafeTable,
        Self::BedRed,
        Self::BedBlue,
        Self::BedGreen,
        Self::BedPink,
        Self::PottedPlant,
        Self::LongPlanter,
        Self::FlowerStand,
        Self::Bookcase,
        Self::LabBookcase,
        Self::Cabinet,
        Self::DrawerCabinet,
        Self::PendulumClock,
        Self::Computer,
        Self::LabWorkstation,
        Self::Television,
        Self::Radio,
        Self::GameConsole,
        Self::Keyboard,
        Self::Refrigerator,
        Self::RetailRefrigerator,
        Self::KitchenStove,
        Self::KitchenSink,
        Self::RetailShelf,
        Self::GiftShelf,
        Self::ReceptionCounter,
        Self::LabCounter,
        Self::HealingMachine,
        Self::HealingConsole,
        Self::ClinicalPartition,
        Self::TimberPartition,
        Self::LabRestorationMachine,
        Self::LabDisplayTable,
        Self::ArcadeMachine,
        Self::ArcadePair,
        Self::PrizeBooth,
        Self::ArcadeWall,
        Self::ArcadePrizeRack,
        Self::OlivineStage,
        Self::StationBench,
        Self::StationTurnstile,
        Self::GateTerminal,
        Self::BroadcastConsole,
        Self::VendingMachine,
        Self::WallPanel,
        Self::WindowWall,
        Self::GateDoorFrame,
        Self::RadioWall,
        Self::OpenBook,
        Self::Memorial,
        Self::BedFeathery,
        Self::BedPolkadot,
        Self::BedPikachu,
        Self::PlantMagna,
        Self::PlantTropic,
        Self::PlantJumbo,
        Self::WallDomestic,
        Self::WallTraditional,
        Self::WallClinical,
        Self::WallAcoustic,
        Self::PictureFrame,
        Self::CarpetCloth,
        Self::StairFlight,
        Self::OfficePhone,
        Self::BroadcastRack,
        Self::TowerReception,
    ];
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Stool => "interior/stool",
            Self::TrashCan => "interior/trash_can",
            Self::ArcadeStool => "interior/arcade_stool",
            Self::Chair => "interior/chair",
            Self::LinkSeat => "interior/link_seat",
            Self::Cushion => "interior/cushion",
            Self::DiningTable => "interior/dining_table",
            Self::LowTable => "interior/low_table",
            Self::CafeTable => "interior/cafe_table",
            Self::BedRed => "interior/bed_red",
            Self::BedBlue => "interior/bed_blue",
            Self::BedGreen => "interior/bed_green",
            Self::BedPink => "interior/bed_pink",
            Self::PottedPlant => "interior/potted_plant",
            Self::LongPlanter => "interior/long_planter",
            Self::FlowerStand => "interior/flower_stand",
            Self::Bookcase => "interior/bookcase",
            Self::LabBookcase => "interior/lab_bookcase",
            Self::Cabinet => "interior/cabinet",
            Self::DrawerCabinet => "interior/drawer_cabinet",
            Self::PendulumClock => "interior/pendulum_clock",
            Self::Computer => "interior/computer",
            Self::LabWorkstation => "interior/lab_workstation",
            Self::Television => "interior/television",
            Self::Radio => "interior/radio",
            Self::GameConsole => "interior/game_console",
            Self::Keyboard => "interior/keyboard",
            Self::Refrigerator => "interior/refrigerator",
            Self::RetailRefrigerator => "interior/retail_refrigerator",
            Self::KitchenStove => "interior/kitchen_stove",
            Self::KitchenSink => "interior/kitchen_sink",
            Self::RetailShelf => "interior/retail_shelf",
            Self::GiftShelf => "interior/gift_shelf",
            Self::ReceptionCounter => "interior/reception_counter",
            Self::LabCounter => "interior/lab_counter",
            Self::HealingMachine => "interior/healing_machine",
            Self::HealingConsole => "interior/healing_console",
            Self::ClinicalPartition => "interior/clinical_partition",
            Self::TimberPartition => "interior/timber_partition",
            Self::LabRestorationMachine => "interior/lab_restoration_machine",
            Self::LabDisplayTable => "interior/lab_display_table",
            Self::ArcadeMachine => "interior/arcade_machine",
            Self::PrizeBooth => "interior/prize_booth",
            Self::ArcadeWall => "interior/arcade_wall",
            Self::ArcadePrizeRack => "interior/arcade_prize_rack",
            Self::OlivineStage => "interior/olivine_stage",
            Self::ArcadePair => "interior/arcade_pair",
            Self::StationBench => "interior/station_bench",
            Self::StationTurnstile => "interior/station_turnstile",
            Self::GateTerminal => "interior/gate_terminal",
            Self::BroadcastConsole => "interior/broadcast_console",
            Self::VendingMachine => "interior/vending_machine",
            Self::WallPanel => "interior/wall_panel",
            Self::WindowWall => "interior/window_wall",
            Self::GateDoorFrame => "interior/gate_door_frame",
            Self::RadioWall => "interior/radio_wall",
            Self::OpenBook => "interior/open_book",
            Self::Memorial => "interior/memorial",
            Self::BedFeathery => "interior/bed_feathery",
            Self::BedPolkadot => "interior/bed_polkadot",
            Self::BedPikachu => "interior/bed_pikachu",
            Self::PlantMagna => "interior/plant_magna",
            Self::PlantTropic => "interior/plant_tropic",
            Self::PlantJumbo => "interior/plant_jumbo",
            Self::WallDomestic => "interior/wall_domestic",
            Self::WallTraditional => "interior/wall_traditional",
            Self::WallClinical => "interior/wall_clinical",
            Self::WallAcoustic => "interior/wall_acoustic",
            Self::CarpetCloth => "interior/carpet_cloth",
            Self::PictureFrame => "interior/picture_frame",
            Self::StairFlight => "interior/stair_flight",
            Self::OfficePhone => "interior/office_phone",
            Self::BroadcastRack => "interior/broadcast_rack",
            Self::TowerReception => "interior/tower_reception",
        }
    }
}
#[derive(Deserialize)]
struct Primitive {
    #[serde(default)]
    name: String,
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
    #[serde(default)]
    preserve_aspect: bool,
    #[serde(default)]
    align_footprint: bool,
    #[serde(default)]
    fit_footprint: bool,
}
struct PaletteRange {
    vertices: std::ops::Range<usize>,
    source: [f32; 3],
    paint: [f32; 3],
}
pub(crate) struct Model {
    surface: crate::shared_model_geometry::ModelSurface,
    room_finishes: Vec<(std::ops::Range<usize>, u8)>,
    room_palette: Vec<PaletteRange>,
    arcade_palette: Vec<PaletteRange>,
    roof_seat_palette: Vec<PaletteRange>,
    olivine_materials: Option<Vec<u8>>,
    store_display_materials: Option<Vec<u8>>,
    store_counter_body_indices: Option<Vec<u32>>,
    native_bookcase_materials: Option<Vec<Option<u8>>>,
    native_gate_counter_materials: Option<Vec<u8>>,
    native_cafe_table_materials: Option<Vec<u8>>,
    native_memorial_materials: Option<Vec<u8>>,
    native_computer_materials: Option<Vec<u8>>,
    native_dining_table_materials: Option<Vec<u8>>,
    native_television_materials: Option<Vec<u8>>,
    native_trashcan_materials: Option<Vec<u8>>,
    min: [f32; 3],
    max: [f32; 3],
    preserve_aspect: bool,
    align_footprint: bool,
    fit_footprint: bool,
}
impl Model {
    pub(crate) fn parse<'a>(
        json: impl Into<crate::model_storage::Source<'a>>,
    ) -> Result<Self, String> {
        let export: Export = crate::model_storage::parse(json)?;
        let mut surface = SurfaceMeshData::default();
        let mut room_finishes = Vec::new();
        let mut room_palette = Vec::new();
        let mut arcade_palette = Vec::new();
        let mut roof_seat_palette = Vec::new();
        let mut olivine_materials=Vec::new();
        let mut olivine_valid=export.name=="olivine_stage";
        let mut olivine_seen=0u8;
        let mut store_display_materials = Vec::new();
        let mut store_display_valid = export.name == "closed_u_display";
        let mut store_display_seen = 0u16;
        let mut counter_body = Vec::new();
        let mut counter_valid = export.name == "reception_counter";
        let mut counter_seen = 0u8;
        let mut native_bookcase_materials = Vec::new();
        let mut native_bookcase_valid = matches!(export.name.as_str(), "bookcase" | "lab_bookcase");
        let mut native_bookcase_seen = 0u8;
        let mut selected_bookcase_structure=false;
        // This exporter retains primitive material ownership in `name`.
        // Restrict it to the exact 16 authored counter modules.
        let counter_mask=export.name.strip_prefix("counter_").filter(|n|n.len()==2)
            .and_then(|n|u8::from_str_radix(n,16).ok()).filter(|n|*n<16);
        let mut native_gate_counter_valid=counter_mask.is_some();
        let mut native_gate_counter_seen=0u8;
        let mut native_gate_counter_materials=Vec::new();
        let mut native_cafe_table_valid=export.name=="cafe_table";
        let mut native_cafe_table_seen=0u8;
        let mut native_cafe_table_materials=Vec::new();
        let mut native_memorial_valid=export.name=="memorial";
        let mut native_memorial_seen=0u8;
        let mut native_memorial_materials=Vec::new();
        let mut native_trashcan_materials=Vec::new();
        let mut native_trashcan_valid=export.name=="gym_bin";
        let mut native_trashcan_seen=0u8;
        let mut native_television_materials=Vec::new();
        let mut native_television_valid=export.name=="television";
        let mut native_television_seen=0u8;
        let mut native_dining_table_materials=Vec::new();
        let mut native_dining_table_valid=export.name=="tableCrossCloth";
        let mut native_dining_table_seen=0u8;
        let mut native_computer_materials=Vec::new();
        let mut native_computer_valid=export.name=="computer";
        let mut native_computer_seen=0u8;
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for p in export.primitives {
            let n = p.positions.len() / 3;
            if n == 0
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
                return Err("invalid interior mesh primitive".into());
            }
            let base = surface.positions.len() as u32;
            if export.name == "reception_counter" {
                let role = match p.material.as_str() {
                    "plaster" => Some(0), "roof_red" => Some(1), "timber" => Some(2),
                    "enamel_light" => Some(3), "timber_light" => Some(4), _ => None,
                };
                if let Some(role) = role {
                    counter_seen |= 1 << role;
                    if role < 3 { counter_body.extend(p.indices.iter().map(|i|i+base)); }
                } else { counter_valid = false; }
            }
            if export.name == "closed_u_display" {
                let material = match p.material.as_str() {
                    "warm_ivory" => Some(0), "countertop_enamel" => Some(1),
                    "ochre_trim" => Some(2), "drawer_face" => Some(3),
                    "blue_glass" => Some(4), "blue_reflection" => Some(5),
                    "blue_glint" => Some(6), "copper" => Some(7),
                    "walnut_plinth" => Some(8), "deep_teal" => Some(9), _ => None,
                };
                if let Some(material) = material {
                    store_display_seen |= 1 << material;
                    store_display_materials.extend(std::iter::repeat_n(material, n));
                } else { store_display_valid = false; }
            }
            if export.name=="olivine_stage" {
                let material=match p.material.as_str() {
                    "deck_stone"=>Some(0),"deck_stone_light"=>Some(1),
                    "retaining_gold"=>Some(2),"retaining_ochre"=>Some(3),
                    "mortar"=>Some(4),"stair_stone"=>Some(5),"stair_lip"=>Some(6),_=>None,
                };
                if let Some(material)=material {
                    olivine_seen|=1<<material;
                    olivine_materials.extend(std::iter::repeat_n(material,n));
                } else {olivine_valid=false;}
            }
            let index_start = surface.indices.len();
            for v in p.positions.chunks_exact(3) {
                let v = [v[0], v[1], v[2]];
                for axis in 0..3 {
                    min[axis] = min[axis].min(v[axis]);
                    max[axis] = max[axis].max(v[axis]);
                }
                surface.positions.push(v);
                surface.uvs.push([0.0, 0.0]);
                surface.colors.push(p.base_color);
            }
            for v in p.normals.chunks_exact(3) {
                let n = Vec3::new(v[0], v[1], v[2]);
                if n.length_squared() < 0.000001 {
                    return Err("zero interior normal".into());
                }
                surface.normals.push(n.normalize().to_array());
            }
            surface
                .indices
                .extend(p.indices.into_iter().map(|i| base + i));
            if !crate::roof_tiles::refine(&mut surface, base as usize, index_start, &p.material) {
                crate::scenery_finish::refine(
                    &mut surface,
                    base as usize,
                    index_start,
                    &p.material,
                    &export.name,
                );
            }
            if matches!(export.name.as_str(), "bookcase" | "lab_bookcase") {
                // Structural ownership follows the authored material names,
                // including any vertices added by surface refinement. Books
                // retain their authored variety; the cabinet uses its live art.
                let role = if export.name=="bookcase" && p.name=="bookcaseClosed(Clone)"
                    && p.material=="wood" {
                    selected_bookcase_structure=true;
                    Some(2)
                } else if export.name=="bookcase" && p.name=="books(Clone)"
                    && matches!(p.material.as_str(),"carpetDarker"|"carpetWhite"|"plant"|"metal") {
                    None
                } else {match p.material.as_str() {
                    "timber" | "linen_shadow" => Some(0),
                    "timber_light" | "enamel" => Some(1),
                    "timber_gold" | "enamel_light" => Some(2),
                    "blue" | "brass" | "leaf" | "purple" | "red" => None,
                    _ => { native_bookcase_valid = false; None },
                }};
                if let Some(role) = role { native_bookcase_seen |= 1 << role; }
                native_bookcase_materials.extend(std::iter::repeat_n(role,
                    surface.positions.len() - base as usize));
            }
            if export.name=="gym_bin" {
                let role=match (p.name.as_str(),p.material.as_str()) {
                    ("ink","")=>Some(0),("metal_light","")=>Some(1),("steel","")=>Some(2),_=>None,
                };
                if let Some(role)=role {
                    native_trashcan_seen|=1<<role;
                    native_trashcan_materials.extend(std::iter::repeat_n(role,surface.positions.len()-base as usize));
                } else {native_trashcan_valid=false;}
            }
            if export.name=="television" {
                let role=match (p.name.as_str(),p.material.as_str()) {
                    ("cabinetTelevision(Clone)","wood")=>Some(0),
                    ("televisionVintage(Clone)","wood")=>Some(1),
                    ("televisionVintage(Clone)","metal")=>Some(2),
                    ("televisionVintage(Clone)","metalMedium")=>Some(3),
                    ("televisionVintage(Clone)","metalDark")=>Some(4),_=>None,
                };
                if let Some(role)=role {
                    native_television_seen|=1<<role;
                    native_television_materials.extend(std::iter::repeat_n(role,
                        surface.positions.len()-base as usize));
                } else {native_television_valid=false;}
            }
            if export.name=="tableCrossCloth" {
                let role=match (p.name.as_str(),p.material.as_str()) {
                    ("tableCrossCloth(Clone)","wood")=>Some(0),
                    ("tableCrossCloth(Clone)","carpet")=>Some(1),_=>None,
                };
                if let Some(role)=role {
                    native_dining_table_seen|=1<<role;
                    native_dining_table_materials.extend(std::iter::repeat_n(role,
                        surface.positions.len()-base as usize));
                } else {native_dining_table_valid=false;}
            }
            if export.name=="computer" {
                let role=match p.material.as_str() {
                    "rubber"|"screen"=>Some(0),
                    "blue"|"brass"|"enamel"|"metal"|"timber_gold"|"timber_light"=>Some(1),
                    "enamel_light"|"screen_light"=>Some(2),_=>None,
                };
                if let Some(role)=role {
                    native_computer_seen|=1<<role;
                    native_computer_materials.extend(std::iter::repeat_n(role,
                        surface.positions.len()-base as usize));
                } else {native_computer_valid=false;}
            }
            if export.name=="memorial" {
                let role=match p.material.as_str() {
                    "stone"=>Some(0),"stone_dark"=>Some(1),"stone_light"=>Some(2),_=>None,
                };
                if let Some(role)=role {
                    native_memorial_seen|=1<<role;
                    native_memorial_materials.extend(std::iter::repeat_n(role,
                        surface.positions.len()-base as usize));
                } else {native_memorial_valid=false;}
            }
            if export.name=="cafe_table" {
                let role=match p.material.as_str() {
                    "linen"=>Some(0),"metal"=>Some(1),"timber_light"=>Some(2),_=>None,
                };
                if let Some(role)=role {
                    native_cafe_table_seen|=1<<role;
                    native_cafe_table_materials.extend(std::iter::repeat_n(role,
                        surface.positions.len()-base as usize));
                } else {native_cafe_table_valid=false;}
            }
            if counter_mask.is_some() {
                let role=match p.name.as_str() {
                    "timber"=>Some(0),"timber_gold"=>Some(1),
                    "timber_light"=>Some(2),"panel"=>Some(3),_=>None,
                };
                if let Some(role)=role {
                    native_gate_counter_seen|=1<<role;
                    native_gate_counter_materials.extend(std::iter::repeat_n(role,
                        surface.positions.len()-base as usize));
                } else {native_gate_counter_valid=false;}
            }
            // Retain named primitive ownership for an explicitly requested store
            // palette. Ordinary house/gate plant placement keeps its authored paint.
            if export.name == "station_bench" {
                // Native roof seating: pink panels and coral trim, in linear
                // color. Station placement does not opt into this treatment.
                let paint = match p.material.as_str() {
                    "timber_gold" => Some([1., 0.332452, 0.564712]),
                    "timber_light" => Some([0.930111, 0.084376, 0.030713]),
                    _ => None,
                };
                if let Some(paint) = paint {
                    roof_seat_palette.push(PaletteRange {
                        vertices: base as usize..surface.positions.len(),
                        source: [p.base_color[0], p.base_color[1], p.base_color[2]], paint,
                    });
                }
            }
            if export.name == "potted_plant" {
                let paint: Option<[f32; 3]> = match p.material.as_str() {
                    "leaf_dark" => Some([0.025, 0.13, 0.015]),
                    "leaf" => Some([0.065, 0.32, 0.02]),
                    "leaf_light" => Some([0.18, 0.48, 0.045]),
                    "leaf_gold" => Some([0.34, 0.60, 0.07]),
                    "roof_red" => Some([0.72, 0.22, 0.08]),
                    "roof_red_light" => Some([0.88, 0.34, 0.12]),
                    _ => None,
                };
                if let Some(paint) = paint {
                    room_palette.push(PaletteRange { vertices: base as usize..surface.positions.len(),
                        source: [p.base_color[0],p.base_color[1],p.base_color[2]], paint });
                }
            }
            if export.name == "cushion" {
                let paint: Option<[f32; 3]> = match p.material.as_str() {
                    "purple" => Some([0.78, 0.19, 0.42]),
                    "linen_shadow" => Some([0.74, 0.27, 0.21]),
                    _ => None,
                };
                if let Some(paint) = paint {
                    room_palette.push(PaletteRange { vertices: base as usize..surface.positions.len(),
                        source: [p.base_color[0],p.base_color[1],p.base_color[2]], paint });
                    room_finishes.push((base as usize..surface.positions.len(), 3));
                }
            }
            if matches!(export.name.as_str(), "arcade_machine" | "arcade_pair") {
                if p.material == "purple" {
                    let paint = [0.89, 0.36, 0.52];
                    arcade_palette.push(PaletteRange { vertices: base as usize..surface.positions.len(),
                        source: [p.base_color[0],p.base_color[1],p.base_color[2]], paint });
                }
                if matches!(p.material.as_str(), "purple" | "red" | "linen" | "yellow") {
                    room_finishes.push((base as usize..surface.positions.len(), 2));
                }
            }
            if export.name == "prize_booth" {
                let kind = match p.material.as_str() {
                    "timber" => Some(1),
                    "blue_enamel" | "green_enamel" | "gold_enamel" => Some(2),
                    _ => None,
                };
                if let Some(kind) = kind { room_finishes.push((base as usize..surface.positions.len(),kind)); }
            }
            if export.name == "arcade_wall" {
                let kind = match p.material.as_str() {
                    "timber" => Some(1),
                    "ochre" | "gold" => Some(2),
                    _ => None,
                };
                if let Some(kind) = kind { room_finishes.push((base as usize..surface.positions.len(),kind)); }
            }
            if export.name == "olivine_stage" && matches!(p.material.as_str(),
                "deck_stone" | "deck_stone_light" | "retaining_gold" | "retaining_ochre") {
                room_finishes.push((base as usize..surface.positions.len(), 2));
            }
            if export.name == "arcade_prize_rack" && matches!(p.material.as_str(),
                "coral_enamel" | "violet_enamel" | "cream_enamel" | "teal_enamel") {
                room_finishes.push((base as usize..surface.positions.len(), 2));
            }
            if export.name == "reception_counter" {
                // Casino service furniture shares geometry with stores, but
                // its opt-in paint follows the source green-and-gold counter.
                let paint: Option<[f32;3]> = match p.material.as_str() {
                    "roof_red" => Some([0.12,0.32,0.012]),
                    "enamel_light" => Some([0.76,0.50,0.14]),
                    _ => None,
                };
                if let Some(paint) = paint {
                    arcade_palette.push(PaletteRange { vertices: base as usize..surface.positions.len(),
                        source: [p.base_color[0],p.base_color[1],p.base_color[2]], paint });
                }
            }
            if export.name == "reception_counter" {
                let kind = match p.material.as_str() {
                    "timber" | "timber_light" => Some(1),
                    "enamel_light" | "roof_red" => Some(2),
                    _ => None,
                };
                if let Some(kind) = kind {
                    room_finishes.push((base as usize..surface.positions.len(),kind));
                }
            }
        }
        crate::scenery_finish::contain(&mut surface, min, max);
        if (0..3).any(|axis| !min[axis].is_finite() || max[axis] <= min[axis]) {
            return Err("empty or flat interior model".into());
        }
        let olivine_materials=(olivine_valid && olivine_seen==0x7f
            && olivine_materials.len()==surface.positions.len()).then_some(olivine_materials);
        let store_display_materials = (store_display_valid && store_display_seen == 0x3ff
            && store_display_materials.len() == surface.positions.len())
            .then_some(store_display_materials);
        let native_bookcase_materials = (native_bookcase_valid
            && native_bookcase_seen == if selected_bookcase_structure {4}else{7}
            && native_bookcase_materials.len() == surface.positions.len())
            .then_some(native_bookcase_materials);
        let native_gate_counter_materials=(native_gate_counter_valid
            && native_gate_counter_seen==if counter_mask==Some(0){7}else{15}
            && native_gate_counter_materials.len()==surface.positions.len())
            .then_some(native_gate_counter_materials);
        let native_cafe_table_materials=(native_cafe_table_valid && native_cafe_table_seen==7
            && native_cafe_table_materials.len()==surface.positions.len())
            .then_some(native_cafe_table_materials);
        let native_memorial_materials=(native_memorial_valid && native_memorial_seen==7
            && native_memorial_materials.len()==surface.positions.len())
            .then_some(native_memorial_materials);
        let native_computer_materials=(native_computer_valid && native_computer_seen==7
            && native_computer_materials.len()==surface.positions.len()).then_some(native_computer_materials);
        let native_dining_table_materials=(native_dining_table_valid && native_dining_table_seen==3
            && native_dining_table_materials.len()==surface.positions.len()).then_some(native_dining_table_materials);
        let native_television_materials=(native_television_valid && native_television_seen==31
            && native_television_materials.len()==surface.positions.len()).then_some(native_television_materials);
        let native_trashcan_materials=(native_trashcan_valid && native_trashcan_seen==7
            && native_trashcan_materials.len()==surface.positions.len()).then_some(native_trashcan_materials);
        Ok(Self {
            native_trashcan_materials,
            surface: surface.into(),
            room_finishes,
            room_palette,
            arcade_palette,
            roof_seat_palette,
            olivine_materials,
            store_display_materials,
            store_counter_body_indices: (counter_valid && counter_seen==0x1f).then_some(counter_body),
            native_bookcase_materials,
            native_gate_counter_materials,
            native_cafe_table_materials,
            native_memorial_materials,
            native_computer_materials,
            native_dining_table_materials,
            native_television_materials,
            min,
            max,
            preserve_aspect: export.preserve_aspect,
            align_footprint: export.align_footprint,
            fit_footprint: export.fit_footprint,
        })
    }
    pub(crate) fn has_native_trashcan_materials(&self)->bool {self.native_trashcan_materials.is_some()}
    pub(crate) fn append_native_trashcan(&self,target:&mut SurfaceMeshData,bounds:[f32;4],height:f32,swatches:[[f32;2];3])->bool {
        let Some(roles)=&self.native_trashcan_materials else {return false;};
        let first=target.positions.len();self.append_fitted(target,bounds,0.,height);
        for (i,&role) in roles.iter().enumerate() {
            target.uvs[first+i]=swatches[role as usize];
            target.colors[first+i]=authored_face_color([1.;4],Vec3::from_array(target.normals[first+i]));
        }
        true
    }
    pub(crate) fn has_native_television_materials(&self)->bool {self.native_television_materials.is_some()}
    pub(crate) fn append_native_television(&self,target:&mut SurfaceMeshData,bounds:[f32;4],
        height:f32,swatches:[[f32;2];5])->bool {
        let Some(roles)=&self.native_television_materials else {return false;};
        let first=target.positions.len();self.append_fitted(target,bounds,0.,height);
        for (i,&role) in roles.iter().enumerate() {
            target.uvs[first+i]=swatches[role as usize];
            target.colors[first+i]=authored_face_color([1.;4],Vec3::from_array(target.normals[first+i]));
        }
        true
    }
    pub(crate) fn append_native_television_screen(&self,target:&mut SurfaceMeshData,bounds:[f32;4],
        height:f32,swatches:[[f32;2];5])->bool {
        let Some(roles)=&self.native_television_materials else {return false;};
        let mut full=SurfaceMeshData::default();
        if !self.append_native_television(&mut full,bounds,height,swatches) {return false;}
        // Reuse the selected CRT's own triangles, excluding its cabinet for
        // a complete source drawing that contains only the screen.
        let mut screen=crate::mesh::select_surface_indices(&full,&vec![false;full.positions.len()],
            &mut vec![u32::MAX;full.positions.len()],full.indices.iter().copied().filter(|i|roles[*i as usize]!=0));
        let min=screen.positions.iter().fold([f32::INFINITY;3],|a,p|std::array::from_fn(|i|a[i].min(p[i])));
        let max=screen.positions.iter().fold([f32::NEG_INFINITY;3],|a,p|std::array::from_fn(|i|a[i].max(p[i])));
        let [w,e,n,s]=bounds;let extent=Vec3::from_array(max)-Vec3::from_array(min);
        let scale=((e-w)/extent.x).min((s-n)/extent.z).min(height/extent.y);
        assert!(scale.is_finite()&&scale>0.,"native screen-only television requires complete CRT geometry");
        let origin=Vec3::new(w+(e-w-extent.x*scale)*0.5,0.,n+(s-n-extent.z*scale)*0.5);
        for p in &mut screen.positions {*p=((Vec3::from_array(*p)-Vec3::from_array(min))*scale+origin).to_array();}
        let base=target.positions.len()as u32;
        target.positions.extend(screen.positions);target.normals.extend(screen.normals);
        target.uvs.extend(screen.uvs);target.colors.extend(screen.colors);
        target.indices.extend(screen.indices.into_iter().map(|i|i+base));
        true
    }

    pub(crate) fn has_native_dining_table_materials(&self)->bool {self.native_dining_table_materials.is_some()}
    pub(crate) fn append_native_dining_table(&self,target:&mut SurfaceMeshData,bounds:[f32;4],
        height:f32,swatches:[[f32;2];2])->bool {
        let Some(roles)=&self.native_dining_table_materials else {return false;};
        let first=target.positions.len();self.append_fitted_with_footprint(target,bounds,0.,height,true);
        for (i,&role) in roles.iter().enumerate() {
            target.uvs[first+i]=swatches[role as usize];
            target.colors[first+i]=authored_face_color([1.;4],Vec3::from_array(target.normals[first+i]));
        }
        true
    }
    pub(crate) fn has_native_computer_materials(&self)->bool {self.native_computer_materials.is_some()}
    pub(crate) fn append_native_computer(&self,target:&mut SurfaceMeshData,bounds:[f32;4],base:f32,
        height:f32,swatches:[[f32;2];3])->bool {
        let Some(roles)=&self.native_computer_materials else {return false;};
        let first=target.positions.len();
        self.append_fitted(target,bounds,base,height);
        for (i,role) in roles.iter().enumerate() {
            target.uvs[first+i]=swatches[*role as usize];
            target.colors[first+i]=authored_face_color([1.;4],Vec3::from_array(target.normals[first+i]));
        }
        true
    }

    /// Opt-in only after this exact model has been appended. Coordinates are
    /// registered to the authored object, so moving the viewport cannot slide
    /// its finish. Unnamed/unknown materials and preceding geometry stay intact.
    pub(crate) fn apply_room_finish(&self, target: &mut SurfaceMeshData, first: usize) {
        assert_eq!(target.positions.len()-first,self.surface.positions.len());
        for (range,kind) in &self.room_finishes {
            for i in range.clone() {
                let p=self.surface.positions[i];
                target.uvs[first+i]=[
                    82000.0+(p[0]-self.min[0])/(self.max[0]-self.min[0])*16.0,
                    (if *kind == 3 { 2500.0 } else { *kind as f32 * 1000.0 })
                        +(p[2]-self.min[2])/(self.max[2]-self.min[2])*16.0,
                ];
            }
        }
    }
    pub(crate) fn apply_arcade_wall_finish(&self, target: &mut SurfaceMeshData, first: usize) {
        self.apply_room_finish(target,first);
        for (range,kind) in &self.room_finishes {
            for i in range.clone() {
                let p=self.surface.positions[i];let n=self.surface.normals[i];
                if n[1].abs()<0.5 {
                    let horizontal=if n[2].abs()>n[0].abs() {
                        (p[0]-self.min[0])/(self.max[0]-self.min[0])*16.
                    } else {
                        (p[2]-self.min[2])/(self.max[2]-self.min[2])*3.
                    };
                    target.uvs[first+i]=[82000.+horizontal,*kind as f32*1000.
                        +(p[1]-self.min[1])/(self.max[1]-self.min[1])*28.];
                }
            }
        }
    }
    /// Explicit store palettes preserve authored grain and fitted face shading.
    /// Unknown materials and all geometry remain unchanged.
    fn apply_palette_ranges(&self, target: &mut SurfaceMeshData, first: usize, ranges: &[PaletteRange]) {
        assert_eq!(target.positions.len() - first, self.surface.positions.len());
        for paint in ranges {
            for i in paint.vertices.clone() {
                let mut color = self.surface.colors[i];
                for axis in 0..3 {
                    // Black is valid authored pigment. An explicitly named
                    // paint retains craft variation even when a channel is zero.
                    color[axis] = if paint.source[axis] > 0.0 {
                        color[axis] * (paint.paint[axis] / paint.source[axis])
                    } else { paint.paint[axis] + color[axis] };
                    color[axis] = color[axis].clamp(0.0,1.0);
                }
                color[3] = target.colors[first+i][3];
                target.colors[first+i] = authored_face_color(color, Vec3::from_array(target.normals[first+i]));
            }
        }
    }
    pub(crate) fn apply_room_palette(&self, target: &mut SurfaceMeshData, first: usize) {
        self.apply_palette_ranges(target, first, &self.room_palette);
    }
    /// Only the dedicated, source-verified roof-seat adapter opts into this paint.
    pub(crate) fn apply_roof_seat_palette(&self, target: &mut SurfaceMeshData, first: usize) {
        self.apply_palette_ranges(target, first, &self.roof_seat_palette);
    }
    pub(crate) fn apply_arcade_finish(&self, target: &mut SurfaceMeshData, first: usize) {
        self.apply_palette_ranges(target, first, &self.arcade_palette);
        self.apply_room_finish(target, first);
    }
    /// Fit the true 3D asset inside a source-verified physical footprint.
    /// Authored assets fit the complete bounds with inverse-transformed normals.
    /// Imported assets may retain their proportions centered inside those bounds.
    pub(crate) fn append_olivine_stage(
        &self,target:&mut SurfaceMeshData,bounds:[f32;4],height:f32,
        source_uv:impl Fn(u8,[f32;3],[f32;3])->[f32;2],
    ) -> bool {
        let Some(materials)=&self.olivine_materials else {return false;};
        let first=target.positions.len();
        self.append_fitted(target,bounds,0.,height);
        for (i,&material) in materials.iter().enumerate() {
            target.uvs[first+i]=source_uv(material,self.surface.positions[i],self.surface.normals[i]);
            target.colors[first+i]=authored_face_color([1.;4],Vec3::from_array(target.normals[first+i]));
        }
        true
    }

    pub(crate) fn append_store_display(
        &self, target: &mut SurfaceMeshData, bounds: [f32; 4], height: f32,
        source_uv: impl Fn(u8) -> [f32; 2],
    ) -> bool {
        let Some(materials) = &self.store_display_materials else { return false; };
        let first = target.positions.len();
        self.append_fitted(target, bounds, 0., height);
        for (i, &material) in materials.iter().enumerate() {
            target.uvs[first + i] = source_uv(material);
            target.colors[first + i] = authored_face_color(
                [1.; 4], Vec3::from_array(target.normals[first + i]),
            );
        }
        true
    }

    /// The canonical store cabinet keeps its body and panel finish; a joined
    /// countertop is authored separately in map space. Unknown roles reject.
    pub(crate) fn append_store_counter_body(&self, target: &mut SurfaceMeshData,
        bounds: [f32;4], height:f32) -> bool {
        let Some(body)=&self.store_counter_body_indices else {return false;};
        let first=target.positions.len() as u32;
        let indices=target.indices.len();
        let body_top=body.iter().map(|&i|self.surface.positions[i as usize][1]).fold(self.min[1],f32::max);
        let body_height=body_top-self.min[1];
        if body_height<=0.000001 || !height.is_finite() || height<=0. {return false;}
        let fitted_height=height*(self.max[1]-self.min[1])/body_height;
        self.append_fitted(target,bounds,0.,fitted_height);
        self.apply_room_finish(target,first as usize);
        target.indices.truncate(indices);
        target.indices.extend(body.iter().map(|i|i+first));
        true
    }
    pub(crate) fn append_fitted(
        &self,
        target: &mut SurfaceMeshData,
        bounds: [f32; 4],
        base_y: f32,
        height: f32,
    ) {
        self.append_fitted_with_footprint(target,bounds,base_y,height,self.fit_footprint);
    }
    // Complete square source tables opt into their actual footprint. Other
    // imported furniture retains its authored fitting policy.
    fn append_fitted_with_footprint(&self,target:&mut SurfaceMeshData,bounds:[f32;4],
        base_y:f32,height:f32,fit_footprint:bool) {
        let [west, east, north, south] = bounds;
        let original = Vec3::from_array(self.max) - Vec3::from_array(self.min);
        // Explicitly opted-in tables may turn to fit a long source drawing.
        // Front-facing fixtures retain their authored orientation by default.
        let direct = ((east - west) / original.x).min((south - north) / original.z);
        let turned = ((east - west) / original.z).min((south - north) / original.x);
        let turn = self.preserve_aspect && self.align_footprint && turned > direct + 0.00001;
        let extent = if turn {
            Vec3::new(original.z, original.y, original.x)
        } else {
            original
        };
        let mut scale = Vec3::new(
            (east - west) / extent.x,
            height / extent.y,
            (south - north) / extent.z,
        );
        let mut origin = Vec3::new(west, base_y, north);
        if self.preserve_aspect && !fit_footprint {
            scale = Vec3::splat(scale.min_element());
            let size = extent * scale;
            origin.x += (east - west - size.x) * 0.5;
            origin.z += (south - north - size.z) * 0.5;
        }
        let base = target.positions.len() as u32;
        for ((p, n), c) in self
            .surface
            .positions
            .iter()
            .zip(&self.surface.normals)
            .zip(&self.surface.colors)
        {
            let mut p = Vec3::from_array(*p) - Vec3::from_array(self.min);
            let mut n = Vec3::from_array(*n);
            if turn {
                p = Vec3::new(p.z, p.y, original.x - p.x);
                n = Vec3::new(n.z, n.y, -n.x);
            }
            let p = p * scale + origin;
            let normal = (n / scale).normalize();
            target.positions.push(p.to_array());
            target.normals.push(normal.to_array());
            target.uvs.push([0.0, 0.0]);
            target.colors.push(authored_face_color(*c, normal));
        }
        target
            .indices
            .extend(self.surface.indices.iter().map(|&i| base + i));
    }

    /// Source palette for an already authenticated cabinet drawing. Unknown
    /// model materials leave the target untouched rather than guessing roles.
    pub(crate) fn append_native_bookcase(
        &self, target: &mut SurfaceMeshData, bounds: [f32; 4], base: f32,
        height: f32, swatches: [[f32; 2]; 3],
    ) -> bool {
        let Some(roles) = &self.native_bookcase_materials else { return false; };
        let first = target.positions.len();
        self.append_fitted(target, bounds, base, height);
        for (i, role) in roles.iter().enumerate() {
            if let Some(role) = role {
                target.uvs[first+i] = swatches[*role as usize];
                let grain=1.+0.08*crate::scenery_finish::timber_grain(self.surface.positions[i]);
                target.colors[first+i] = authored_face_color([grain,grain,grain,1.],
                    Vec3::from_array(target.normals[first+i]));
            }
        }
        true
    }
    pub(crate) fn append_native_memorial(&self,target:&mut SurfaceMeshData,
        bounds:[f32;4],base:f32,height:f32,swatches:[[f32;2];3])->bool {
        let Some(roles)=&self.native_memorial_materials else{return false;};
        let first=target.positions.len();self.append_fitted(target,bounds,base,height);
        for (i,role) in roles.iter().enumerate() {
            target.uvs[first+i]=swatches[*role as usize];
            target.colors[first+i]=authored_face_color([1.;4],
                Vec3::from_array(target.normals[first+i]));
        }
        true
    }
    pub(crate) fn append_native_cafe_table(&self,target:&mut SurfaceMeshData,
        bounds:[f32;4],height:f32,swatches:[[f32;2];3])->bool {
        let Some(roles)=&self.native_cafe_table_materials else{return false;};
        let first=target.positions.len();self.append_fitted(target,bounds,0.,height);
        for (i,role) in roles.iter().enumerate() {
            target.uvs[first+i]=swatches[*role as usize];
            let local=self.surface.positions[i];
            let weave=if *role==0 {1.+0.035*(local[0]*53.).sin()*(local[2]*47.).sin()}else{1.};
            target.colors[first+i]=authored_face_color([weave,weave,weave,1.],
                Vec3::from_array(target.normals[first+i]));
        }
        true
    }
    pub(crate) fn has_native_gate_counter_materials(&self) -> bool {
        self.native_gate_counter_materials.is_some()
    }
    pub(crate) fn append_native_gate_counter(&self,target:&mut SurfaceMeshData,
        bounds:[f32;4],height:f32,swatches:[[f32;2];4]) -> bool {
        let Some(roles)=&self.native_gate_counter_materials else {return false;};
        let first=target.positions.len();
        self.append_fitted(target,bounds,0.,height);
        for (i,role) in roles.iter().enumerate() {
            target.uvs[first+i]=swatches[*role as usize];
            let grain=1.+0.06*crate::scenery_finish::timber_grain(self.surface.positions[i]);
            target.colors[first+i]=authored_face_color([grain,grain,grain,1.],
                Vec3::from_array(target.normals[first+i]));
        }
        true
    }
    /// Shared counter topology with a room-specific work surface. Named trim
    /// keeps its own paint; only the actual upward cap receives the top palette.
    pub(crate) fn append_native_service_counter(&self,target:&mut SurfaceMeshData,bounds:[f32;4],
        height:f32,wood:[[f32;2];4],top:[f32;2])->bool {
        let first=target.positions.len();
        if !self.append_native_gate_counter(target,bounds,height,wood){return false;}
        let roles=self.native_gate_counter_materials.as_ref().unwrap();
        // A separate raised grain primitive lies slightly above the cap.
        // Derive the cap datum from its named upward material faces.
        let cap=roles.iter().enumerate().filter(|&(i,&role)|role==1&&self.surface.normals[i][1]>0.9)
            .map(|(i,_)|self.surface.positions[i][1]).fold(f32::NEG_INFINITY,f32::max);
        for (i,&role) in roles.iter().enumerate() {
            if role==1 && self.surface.positions[i][1]>=cap-1e-5
                && self.surface.normals[i][1]>0.9 {
                target.uvs[first+i]=top;
            }
        }
        true
    }
    pub(crate) fn native_bookcase_structural_mask(&self) -> Option<Vec<bool>> {
        self.native_bookcase_materials.as_ref().map(|roles|
            roles.iter().map(Option::is_some).collect())
    }
}

/// Original model palettes need one face-light bake before the terrain shader,
/// which intentionally adds only shadow visibility to already shaded colors.
/// Call after normal fitting; legacy source terrain keeps its own baked shade.
#[inline]
pub(crate) fn authored_face_color(color: [f32; 4], normal: Vec3) -> [f32; 4] {
    let light = Vec3::new(-0.35, 0.85, 0.40).normalize();
    let shade = 0.64 + 0.36 * normal.dot(light).max(0.0);
    [
        color[0] * shade,
        color[1] * shade,
        color[2] * shade,
        color[3],
    ]
}

#[cfg(test)]
mod native_bookcase_tests {
    use super::*;
    #[test]
    fn selected_bookcase_requires_named_cabinet_and_book_material_ownership() {
        let source=include_str!("../../../assets/open-models/interiors/bookcase.mesh.json");
        for changed in [source.replace("bookcaseClosed(Clone)","unknown_body"),
            source.replace("books(Clone)","unknown_books"),source.replace("carpetDarker","unknown_ink")] {
            let changed=Model::parse(changed.as_str()).unwrap();
            assert!(changed.native_bookcase_structural_mask().is_none());
            let mut out=SurfaceMeshData::default();
            assert!(!changed.append_native_bookcase(&mut out,[0.,16.,0.,8.],0.,29.,[[0.;2];3]));
            assert!(out.positions.is_empty());
        }
    }
    #[test]
    fn kanto_native_bookcase_palette_keeps_geometry_books_and_cached_model() {
        for kind in [ModelKind::Bookcase,ModelKind::LabBookcase] {
            let cabinet=model(kind);
            let mut original=SurfaceMeshData::default();
            cabinet.append_fitted(&mut original,[2.,18.,3.,12.],4.,29.);
            let mut native=SurfaceMeshData::default();
            let swatches=[[0.1,0.2],[0.3,0.4],[0.5,0.6]];
            assert!(cabinet.append_native_bookcase(&mut native,[2.,18.,3.,12.],4.,29.,swatches));
            assert_eq!(native.positions,original.positions);
            assert_eq!(native.normals,original.normals);
            assert_eq!(native.indices,original.indices);
            let roles=cabinet.native_bookcase_materials.as_ref().unwrap();
            for (i,role) in roles.iter().enumerate() {
                if let Some(role)=role {assert_eq!(native.uvs[i],swatches[*role as usize]);}
                else {assert_eq!(native.colors[i],original.colors[i]);assert_eq!(native.uvs[i],[0.,0.]);}
            }
            let mut cached=SurfaceMeshData::default();
            cabinet.append_fitted(&mut cached,[2.,18.,3.,12.],4.,29.);
            assert_eq!(cached.colors,original.colors);assert_eq!(cached.uvs,original.uvs);
        }
        let mut untouched=SurfaceMeshData::default();
        assert!(!model(ModelKind::Chair).append_native_bookcase(&mut untouched,[0.,8.,0.,8.],0.,8.,[[0.1,0.2];3]));
        assert!(untouched.positions.is_empty());
    }
}

#[cfg(test)]
mod face_shading_tests {
    use super::*;

    #[test]
    fn shared_bake_preserves_existing_interior_response_and_alpha() {
        let color = [0.4, 0.6, 0.8, 0.37];
        let light = Vec3::new(-0.35, 0.85, 0.40).normalize();
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let n = Vec3::new(x as f32, y as f32, z as f32);
                    if n == Vec3::ZERO {
                        continue;
                    }
                    let n = n.normalize();
                    let shade = 0.64 + 0.36 * n.dot(light).max(0.0);
                    let previous = [
                        color[0] * shade,
                        color[1] * shade,
                        color[2] * shade,
                        color[3],
                    ];
                    assert_eq!(
                        authored_face_color(color, n).map(f32::to_bits),
                        previous.map(f32::to_bits)
                    );
                }
            }
        }
        assert!(authored_face_color(color, Vec3::Y)[0] > authored_face_color(color, -Vec3::Y)[0]);
    }
}
/// The five external furniture families have one selected source. Missing or
/// invalid selected art is missing coverage, never a different embedded model.
pub(crate) fn selected_model(kind: ModelKind) -> Option<&'static Model> {
    macro_rules! required {
        ($slot:ident, $path:literal) => {{
            static $slot: OnceLock<Option<Model>> = OnceLock::new();
            $slot.get_or_init(|| crate::open_models::load_required($path,
                |source| Model::parse(source))).as_ref()
        }};
    }
    match kind {
        ModelKind::Bookcase => required!(BOOKCASE, "models/interiors/bookcase.mesh.json"),
        ModelKind::DiningTable => required!(DINING_TABLE, "models/interiors/dining_table.mesh.json"),
        ModelKind::BedRed => required!(BED_RED, "models/interiors/bed_red.mesh.json"),
        ModelKind::Chair => required!(CHAIR, "models/interiors/chair.mesh.json"),
        ModelKind::Television => required!(TELEVISION, "models/interiors/television.mesh.json"),
        _ => Some(model(kind)),
    }
}

/// Direct authored catalog access. This never substitutes for a missing
/// selected external family; production furnishing claims use selected_model.
pub(crate) fn model(kind: ModelKind) -> &'static Model {
    macro_rules! load {
        ($slot:ident, $path:literal) => {{
            static $slot: OnceLock<Model> = OnceLock::new();
            $slot.get_or_init(|| {
                Model::parse(crate::model_storage::include_model!($path))
                    .expect("validated designated interior asset")
            })
        }};
    }
    match kind {
        ModelKind::TrashCan => load!(TRASH_CAN, "models/dungeons/gym_bin.mesh.json"),
        ModelKind::PrizeBooth => load!(PRIZE_BOOTH, "models/interiors/prize_booth.mesh.json"),
        ModelKind::ArcadeWall => load!(ARCADE_WALL, "models/interiors/arcade_wall.mesh.json"),
        ModelKind::ArcadePrizeRack => load!(ARCADE_PRIZE_RACK, "models/interiors/arcade_prize_rack.mesh.json"),
        ModelKind::OlivineStage => load!(OLIVINE_STAGE, "models/interiors/olivine_stage.mesh.json"),
        ModelKind::BroadcastRack => {
            load!(BROADCAST_RACK, "models/interiors/broadcast_rack.mesh.json")
        }
        ModelKind::TowerReception => load!(
            TOWER_RECEPTION,
            "models/interiors/tower_reception.mesh.json"
        ),
        ModelKind::Stool => load!(STOOL, "models/interiors/stool.mesh.json"),
        ModelKind::ArcadeStool => load!(ARCADE_STOOL, "models/interiors/arcade_stool.mesh.json"),
        ModelKind::Chair => selected_model(kind).expect("required selected interior asset missing"),
        ModelKind::LinkSeat => load!(LINK_SEAT, "models/interiors/link_seat.mesh.json"),
        ModelKind::Cushion => load!(CUSHION, "models/interiors/cushion.mesh.json"),
        ModelKind::DiningTable => selected_model(kind).expect("required selected interior asset missing"),
        ModelKind::LowTable => load!(LOW_TABLE, "models/interiors/low_table.mesh.json"),
        ModelKind::CafeTable => load!(CAFE_TABLE, "models/interiors/cafe_table.mesh.json"),
        ModelKind::BedRed => selected_model(kind).expect("required selected interior asset missing"),
        ModelKind::BedBlue => load!(BED_BLUE, "models/interiors/bed_blue.mesh.json"),
        ModelKind::BedGreen => load!(BED_GREEN, "models/interiors/bed_green.mesh.json"),
        ModelKind::BedPink => load!(BED_PINK, "models/interiors/bed_pink.mesh.json"),
        ModelKind::PottedPlant => load!(POTTED_PLANT, "models/interiors/potted_plant.mesh.json"),
        ModelKind::LongPlanter => load!(LONG_PLANTER, "models/interiors/long_planter.mesh.json"),
        ModelKind::FlowerStand => load!(FLOWER_STAND, "models/interiors/flower_stand.mesh.json"),
        ModelKind::Bookcase => selected_model(kind).expect("required selected interior asset missing"),
        ModelKind::LabBookcase => load!(LAB_BOOKCASE, "models/interiors/lab_bookcase.mesh.json"),
        ModelKind::Cabinet => load!(CABINET, "models/interiors/cabinet.mesh.json"),
        ModelKind::DrawerCabinet => {
            load!(DRAWER_CABINET, "models/interiors/drawer_cabinet.mesh.json")
        }
        ModelKind::PendulumClock => {
            load!(PENDULUM_CLOCK, "models/interiors/pendulum_clock.mesh.json")
        }
        ModelKind::Computer => load!(COMPUTER, "models/interiors/computer.mesh.json"),
        ModelKind::LabWorkstation => load!(
            LAB_WORKSTATION,
            "models/interiors/lab_workstation.mesh.json"
        ),
        ModelKind::Television => selected_model(kind).expect("required selected interior asset missing"),
        ModelKind::Radio => load!(RADIO, "models/interiors/radio.mesh.json"),
        ModelKind::GameConsole => load!(GAME_CONSOLE, "models/interiors/game_console.mesh.json"),
        ModelKind::Keyboard => load!(KEYBOARD, "models/interiors/keyboard.mesh.json"),
        ModelKind::Refrigerator => {
            load!(REFRIGERATOR, "models/interiors/refrigerator.mesh.json")
        }
        ModelKind::RetailRefrigerator => load!(
            RETAIL_REFRIGERATOR,
            "models/interiors/retail_refrigerator.mesh.json"
        ),
        ModelKind::KitchenStove => {
            load!(KITCHEN_STOVE, "models/interiors/kitchen_stove.mesh.json")
        }
        ModelKind::KitchenSink => load!(KITCHEN_SINK, "models/interiors/kitchen_sink.mesh.json"),
        ModelKind::RetailShelf => load!(RETAIL_SHELF, "models/interiors/retail_shelf.mesh.json"),
        ModelKind::GiftShelf => load!(GIFT_SHELF, "models/interiors/gift_shelf.mesh.json"),
        ModelKind::ReceptionCounter => load!(
            RECEPTION_COUNTER,
            "models/interiors/reception_counter.mesh.json"
        ),
        ModelKind::LabCounter => load!(LAB_COUNTER, "models/interiors/lab_counter.mesh.json"),
        ModelKind::HealingMachine => load!(
            HEALING_MACHINE,
            "models/interiors/healing_machine.mesh.json"
        ),
        ModelKind::HealingConsole => load!(
            HEALING_CONSOLE,
            "models/interiors/healing_console.mesh.json"
        ),
        ModelKind::ClinicalPartition => load!(
            CLINICAL_PARTITION,
            "models/interiors/clinical_partition.mesh.json"
        ),
        ModelKind::TimberPartition => load!(
            TIMBER_PARTITION,
            "models/interiors/timber_partition.mesh.json"
        ),
        ModelKind::LabRestorationMachine => load!(
            LAB_RESTORATION_MACHINE,
            "models/interiors/lab_restoration_machine.mesh.json"
        ),
        ModelKind::LabDisplayTable => load!(
            LAB_DISPLAY_TABLE,
            "models/interiors/lab_display_table.mesh.json"
        ),
        ModelKind::ArcadeMachine => {
            load!(ARCADE_MACHINE, "models/interiors/arcade_machine.mesh.json")
        }
        ModelKind::ArcadePair => load!(ARCADE_PAIR, "models/interiors/arcade_pair.mesh.json"),
        ModelKind::StationBench => {
            load!(STATION_BENCH, "models/interiors/station_bench.mesh.json")
        }
        ModelKind::StationTurnstile => load!(
            STATION_TURNSTILE,
            "models/interiors/station_turnstile.mesh.json"
        ),
        ModelKind::GateTerminal => {
            load!(GATE_TERMINAL, "models/interiors/gate_terminal.mesh.json")
        }
        ModelKind::BroadcastConsole => load!(
            BROADCAST_CONSOLE,
            "models/interiors/broadcast_console.mesh.json"
        ),
        ModelKind::VendingMachine => load!(
            VENDING_MACHINE,
            "models/interiors/vending_machine.mesh.json"
        ),
        ModelKind::WallPanel => load!(WALL_PANEL, "models/interiors/wall_panel.mesh.json"),
        ModelKind::WindowWall => load!(WINDOW_WALL, "models/interiors/window_wall.mesh.json"),
        ModelKind::GateDoorFrame => load!(
            GATE_DOOR_FRAME,
            "models/interiors/gate_door_frame.mesh.json"
        ),
        ModelKind::RadioWall => load!(RADIO_WALL, "models/interiors/radio_wall.mesh.json"),
        ModelKind::OpenBook => load!(OPEN_BOOK, "models/interiors/open_book.mesh.json"),
        ModelKind::Memorial => load!(MEMORIAL, "models/interiors/memorial.mesh.json"),
        ModelKind::BedFeathery => load!(BED_FEATHERY, "models/interiors/bed_feathery.mesh.json"),
        ModelKind::BedPolkadot => load!(BED_POLKADOT, "models/interiors/bed_polkadot.mesh.json"),
        ModelKind::BedPikachu => load!(BED_PIKACHU, "models/interiors/bed_pikachu.mesh.json"),
        ModelKind::PlantMagna => load!(PLANT_MAGNA, "models/interiors/plant_magna.mesh.json"),
        ModelKind::PlantTropic => load!(PLANT_TROPIC, "models/interiors/plant_tropic.mesh.json"),
        ModelKind::PlantJumbo => load!(PLANT_JUMBO, "models/interiors/plant_jumbo.mesh.json"),
        ModelKind::WallDomestic => {
            load!(WALL_DOMESTIC, "models/interiors/wall_domestic.mesh.json")
        }
        ModelKind::WallTraditional => load!(
            WALL_TRADITIONAL,
            "models/interiors/wall_traditional.mesh.json"
        ),
        ModelKind::WallClinical => {
            load!(WALL_CLINICAL, "models/interiors/wall_clinical.mesh.json")
        }
        ModelKind::WallAcoustic => {
            load!(WALL_ACOUSTIC, "models/interiors/wall_acoustic.mesh.json")
        }
        ModelKind::CarpetCloth => load!(CARPET_CLOTH, "models/interiors/carpet_cloth.mesh.json"),
        ModelKind::PictureFrame => {
            load!(PICTURE_FRAME, "models/interiors/picture_frame.mesh.json")
        }
        ModelKind::StairFlight => load!(STAIR_FLIGHT, "models/interiors/stair_flight.mesh.json"),
        ModelKind::OfficePhone => load!(OFFICE_PHONE, "models/interiors/office_phone.mesh.json"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kanto_native_screen_only_television_reuses_crt_without_cabinet_triangles() {
        let tv=Model::parse(include_str!("../../../assets/open-models/interiors/television.mesh.json")).unwrap();
        let swatches=[[0.1,0.2],[0.2,0.3],[0.3,0.4],[0.4,0.5],[0.5,0.6]];
        let mut screen=SurfaceMeshData::default();
        assert!(tv.append_native_television_screen(&mut screen,[0.,16.,0.,9.],20.,swatches));
        assert!(!screen.uvs.contains(&swatches[0]),"stand art is absent from screen-only source");
        let roles=tv.native_television_materials.as_ref().unwrap();
        assert_eq!(screen.indices.len(),tv.surface.indices.iter().filter(|i|roles[**i as usize]!=0).count());
        assert!(screen.positions.iter().all(|p|p[1]>=0.&&p[0]>=0.&&p[0]<=16.&&p[2]>=0.&&p[2]<=9.));
        for tri in screen.indices.chunks_exact(3) {
            let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(screen.positions[i as usize]));
            let normal=Vec3::from_array(screen.normals[tri[0]as usize]);
            assert!((b-a).cross(c-a).dot(normal)>=-0.00001);
        }
    }

    #[test]
    fn kanto_native_service_counter_keeps_green_top_off_vertical_gold_trim() {
        for mask in 0..16 {
            let model=crate::gate_counter_models::segment(mask);
            let wood=[[0.1,0.2],[0.3,0.4],[0.5,0.6],[0.7,0.8]];let top=[0.9,0.95];
            let mut base=SurfaceMeshData::default();let mut cafe=SurfaceMeshData::default();
            assert!(model.append_native_gate_counter(&mut base,[0.,8.,0.,8.],8.,wood));
            assert!(model.append_native_service_counter(&mut cafe,[0.,8.,0.,8.],8.,wood,top));
            assert_eq!(base.positions,cafe.positions);assert_eq!(base.indices,cafe.indices);assert_eq!(base.normals,cafe.normals);
            assert!(cafe.uvs.contains(&top));
            for (i,uv) in cafe.uvs.iter().enumerate() {
                if *uv==top {assert!(cafe.positions[i][1]>=7.99&&cafe.normals[i][1]>0.9);}
                else {assert_eq!(*uv,base.uvs[i]);}
            }
        }
    }

    #[test]
    fn kanto_native_trashcan_reuses_gym_geometry_with_separate_source_palette() {
        let bin=model(ModelKind::TrashCan);
        let gym=crate::dungeon_models::model(crate::dungeon_models::Kind::GymBin);
        assert!(std::sync::Arc::ptr_eq(&bin.surface.geometry,gym.shared_geometry()));
        let mut out=SurfaceMeshData::default();let swatches=[[0.1,0.2],[0.3,0.4],[0.5,0.6]];
        assert!(bin.append_native_trashcan(&mut out,[3.,13.,6.,16.],8.,swatches));
        assert!(out.positions.iter().all(|p|p[0]>=3.-1e-5&&p[0]<=13.+1e-5&&p[1]>=0.&&p[1]<=8.+1e-5&&p[2]>=6.-1e-5&&p[2]<=16.+1e-5));
        assert!(out.uvs.iter().all(|uv|swatches.contains(uv)));
        let text=include_str!("../models/dungeons/gym_bin.mesh.json").replace("metal_light","unknown_part");
        let unknown=Model::parse(text.as_str()).unwrap();let mut rejected=SurfaceMeshData::default();
        assert!(!unknown.append_native_trashcan(&mut rejected,[0.,10.,0.,10.],8.,swatches));
        assert!(rejected.positions.is_empty());
    }

    #[test]
    fn kanto_native_television_keeps_selected_crt_geometry_and_rejects_unknown_roles() {
        let source=include_str!("../../../assets/open-models/interiors/television.mesh.json");
        let tv=Model::parse(source).unwrap();let before=tv.surface.clone();
        let swatches=[[0.1,0.2],[0.2,0.3],[0.3,0.4],[0.4,0.5],[0.5,0.6]];
        let mut native=SurfaceMeshData::default();let mut original=SurfaceMeshData::default();
        assert!(tv.append_native_television(&mut native,[0.,16.,0.,9.],20.,swatches));
        tv.append_fitted(&mut original,[0.,16.,0.,9.],0.,20.);
        assert_eq!(native.positions,original.positions);assert_eq!(native.indices,original.indices);
        assert_eq!(native.normals,original.normals);assert_eq!(tv.surface,before);
        for uv in swatches {assert!(native.uvs.contains(&uv));}
        assert!(native.colors.iter().all(|c|c[0]==c[1]&&c[1]==c[2]));
        for altered in [source.replace("cabinetTelevision(Clone)","unknown_cabinet"),
            source.replace("metalMedium","unknown_screen")] {
            let unknown=Model::parse(altered.as_str()).unwrap();let mut rejected=SurfaceMeshData::default();
            assert!(!unknown.append_native_television(&mut rejected,[0.,16.,0.,9.],20.,swatches));
            assert!(rejected.positions.is_empty());
        }
    }

    #[test]
    fn kanto_native_dining_table_keeps_selected_geometry_and_rejects_unknown_roles() {
        let source=include_str!("../../../assets/open-models/interiors/dining_table.mesh.json");
        let table=Model::parse(source).unwrap();let before=table.surface.clone();
        let swatches=[[0.2,0.3],[0.4,0.5]];
        let mut native=SurfaceMeshData::default();let mut original=SurfaceMeshData::default();
        assert!(table.append_native_dining_table(&mut native,[0.,32.,0.,32.],10.,swatches));
        table.append_fitted(&mut original,[0.,32.,0.,32.],0.,10.);
        assert_eq!(native.indices,original.indices);
        for axis in [0,2] {
            let min=native.positions.iter().map(|p|p[axis]).fold(f32::INFINITY,f32::min);
            let max=native.positions.iter().map(|p|p[axis]).fold(f32::NEG_INFINITY,f32::max);
            assert!(min.abs()<0.001 && (max-32.).abs()<0.001,"square native table fills its actual source footprint");
        }
        for ids in native.indices.chunks_exact(3) {
            let [a,b,c]=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(native.positions[i as usize]));
            let normal=Vec3::from_array(native.normals[ids[0]as usize]);
            assert!((b-a).cross(c-a).dot(normal)>=-0.00001);
        }
        assert_eq!(table.surface,before);for uv in swatches {assert!(native.uvs.contains(&uv));}
        assert!(native.colors.iter().all(|c|c[0]==c[1] && c[1]==c[2]));
        let unknown=Model::parse(source.replace("\"carpet\"","\"unknown_cloth\"").as_str()).unwrap();
        let mut rejected=SurfaceMeshData::default();
        assert!(!unknown.append_native_dining_table(&mut rejected,[0.,32.,0.,32.],10.,swatches));
        assert!(rejected.positions.is_empty());
    }
    #[test]
    fn kanto_native_computer_samples_own_palette_and_rejects_unknown_materials() {
        let pc=model(ModelKind::Computer);
        let before=pc.surface.clone();
        let mut native=SurfaceMeshData::default();let mut original=SurfaceMeshData::default();
        let swatches=[[0.1,0.2],[0.3,0.4],[0.5,0.6]];
        assert!(pc.append_native_computer(&mut native,[0.,16.,0.,12.],0.,22.,swatches));
        pc.append_fitted(&mut original,[0.,16.,0.,12.],0.,22.);
        assert_eq!(native.positions,original.positions);assert_eq!(native.indices,original.indices);
        assert_eq!(native.normals,original.normals);assert_eq!(pc.surface,before);
        for uv in swatches {assert!(native.uvs.contains(&uv));}
        assert!(native.colors.iter().all(|c|c[0]==c[1] && c[1]==c[2]));
        let source=include_str!("../models/interiors/computer.mesh.json");
        let unknown=Model::parse(source.replace("screen_light","unknown_screen").as_str()).unwrap();
        let mut rejected=SurfaceMeshData::default();
        assert!(!unknown.append_native_computer(&mut rejected,[0.,16.,0.,12.],0.,22.,swatches));
        assert!(rejected.positions.is_empty());
    }
    #[test]
    fn shared_assets_bed_variants_reuse_geometry_and_keep_their_own_colors() {
        for (a,b) in [(ModelKind::BedBlue,ModelKind::BedGreen)] {
            let a=model(a);let b=model(b);
            assert!(std::sync::Arc::ptr_eq(&a.surface.geometry,&b.surface.geometry));
            assert_ne!(a.surface.colors,b.surface.colors);
            let mut first=SurfaceMeshData::default();let mut second=SurfaceMeshData::default();
            a.append_fitted(&mut first,[0.,16.,0.,16.],0.,10.);
            b.append_fitted(&mut second,[0.,16.,0.,16.],0.,10.);
            assert_eq!(first.positions,second.positions);assert_eq!(first.indices,second.indices);
            assert_ne!(first.colors,second.colors);
        }
    }

    #[test]
    fn kanto_native_memorial_materials_require_all_named_roles_and_keep_cached_geometry() {
        let memorial=model(ModelKind::Memorial);let cached=memorial.surface.clone();
        let mut original=SurfaceMeshData::default();
        memorial.append_fitted(&mut original,[0.,16.,0.,16.],5.,14.);
        let mut native=SurfaceMeshData::default();
        let swatches=[[0.1,0.2],[0.3,0.4],[0.5,0.6]];
        assert!(memorial.append_native_memorial(&mut native,[0.,16.,0.,16.],5.,14.,swatches));
        assert_eq!(native.positions,original.positions);
        assert_eq!(native.normals,original.normals);
        assert_eq!(native.indices,original.indices);
        assert_eq!(memorial.surface,cached);
        for swatch in swatches {assert!(native.uvs.contains(&swatch));}
        let json=include_str!("../models/interiors/memorial.mesh.json");
        let changed=json.replace("stone_light","unknown_memorial_material");
        let changed=Model::parse(changed.as_str()).unwrap();
        let mut rejected=SurfaceMeshData::default();
        assert!(!changed.append_native_memorial(&mut rejected,[0.,16.,0.,16.],5.,14.,swatches));
        assert!(rejected.positions.is_empty());
        assert!(!model(ModelKind::Computer).append_native_memorial(&mut rejected,[0.,16.,0.,16.],5.,14.,swatches));
        assert!(rejected.positions.is_empty());
    }

    #[test]
    fn olivine_stage_native_materials_preserve_geometry_and_reject_unknown_roles() {
        let stage=model(ModelKind::OlivineStage);let cached=stage.surface.clone();
        let mut original=SurfaceMeshData::default();
        stage.append_fitted(&mut original,[0.,96.,0.,64.],0.,8.);
        let mut native=SurfaceMeshData::default();
        assert!(stage.append_olivine_stage(&mut native,[0.,96.,0.,64.],8.,
            |role,_,_|[role as f32/8.,0.5]));
        assert_eq!(native.positions,original.positions);
        assert_eq!(native.normals,original.normals);
        assert_eq!(native.indices,original.indices);
        for role in 0..7 {assert!(native.uvs.contains(&[role as f32/8.,0.5]));}
        assert_eq!(stage.surface,cached);
        for (n,c) in native.normals.iter().zip(&native.colors) {
            assert_eq!(*c,authored_face_color([1.;4],Vec3::from_array(*n)));
        }
        let mut rejected=SurfaceMeshData::default();
        assert!(!model(ModelKind::Computer).append_olivine_stage(&mut rejected,[0.,16.,0.,16.],8.,|_,_,_|[0.;2]));
        assert!(rejected.positions.is_empty());
        let json=include_str!("../models/interiors/olivine_stage.mesh.json");
        let unknown=Model::parse(json.replace("\"stair_lip\"","\"unknown\"").as_str()).unwrap();
        assert!(!unknown.append_olivine_stage(&mut rejected,[0.,96.,0.,64.],8.,|_,_,_|[0.;2]));
        assert!(rejected.positions.is_empty());
    }
    #[test]
    fn roof_seat_named_pink_palette_preserves_station_mesh_and_metal() {
        let source = model(ModelKind::StationBench);
        let cached = source.surface.clone();
        let mut station = SurfaceMeshData::default();
        source.append_fitted(&mut station, [-8., 8., -11., 11.], 0., 8.);
        let mut roof = station.clone();
        source.apply_roof_seat_palette(&mut roof, 0);
        assert_eq!(station.positions, roof.positions);
        assert_eq!(station.normals, roof.normals);
        assert_eq!(station.indices, roof.indices);
        assert_eq!(station.uvs, roof.uvs);
        assert_eq!(source.roof_seat_palette.len(), 2);
        assert_ne!(station.colors, roof.colors);
        for i in 0..station.colors.len() {
            if !source.roof_seat_palette.iter().any(|p| p.vertices.contains(&i)) {
                assert_eq!(station.colors[i], roof.colors[i]);
            }
        }
        assert_eq!(source.surface, cached);
        let json = include_str!("../models/interiors/station_bench.mesh.json");
        assert!(Model::parse(json.replace("station_bench", "other").as_str())
            .unwrap().roof_seat_palette.is_empty());
        assert_eq!(Model::parse(json.replace("timber_gold", "unknown").as_str())
            .unwrap().roof_seat_palette.len(), 1);
    }
    #[test]
    fn department_store_counter_body_requires_complete_named_roles_without_mutating_cache() {
        let json=include_str!("../models/interiors/reception_counter.mesh.json");
        let counter=Model::parse(json).unwrap();let cached=counter.surface.clone();
        let mut mesh=SurfaceMeshData::default();
        assert!(counter.append_store_counter_body(&mut mesh,[0.,16.,0.,16.],9.));
        assert!(mesh.indices.len()<cached.indices.len());
        let top=mesh.indices.iter().map(|&i|mesh.positions[i as usize][1]).fold(0.,f32::max);
        assert!((top-9.).abs()<0.001);
        assert_eq!(counter.surface,cached);
        for role in ["plaster","roof_red","timber","enamel_light","timber_light","reception_counter"] {
            let unknown=Model::parse(json.replace(&format!("\"{role}\""),"\"unknown\"").as_str()).unwrap();
            let before=mesh.clone();
            assert!(!unknown.append_store_counter_body(&mut mesh,[0.,16.,0.,16.],9.));
            assert_eq!(mesh,before,"unknown role must reject before writing");
        }
    }
    #[test]
    fn department_store_display_native_palette_requires_all_named_materials() {
        let json = include_str!("../models/department_store/closed_u_display.mesh.json");
        let display = Model::parse(json).unwrap();
        let cached = display.surface.clone();
        assert!(display.room_finishes.is_empty());
        let mut plain = SurfaceMeshData::default();
        display.append_fitted(&mut plain, [0., 64., 0., 64.], 0., 8.);
        let mut mesh = SurfaceMeshData::default();
        assert!(display.append_store_display(&mut mesh, [0.,64.,0.,64.],8.,
            |material| [material as f32, 0.]));
        assert_eq!(mesh.positions, plain.positions);
        assert_eq!(mesh.normals, plain.normals);
        assert_eq!(mesh.indices, plain.indices);
        for material in 0..10 { assert!(mesh.uvs.contains(&[material as f32, 0.])); }
        assert_eq!(display.surface, cached);
        for changed in [json.replace("countertop_enamel", "unknown_countertop"),
                        json.replace("blue_glass", "unknown_glass"),
                        json.replace("closed_u_display", "other_fixture")] {
            let unknown = Model::parse(changed.as_str()).unwrap();
            assert!(unknown.store_display_materials.is_none());
            let before = mesh.clone();
            assert!(!unknown.append_store_display(&mut mesh,[0.,64.,0.,64.],8.,|_|[0.;2]));
            assert_eq!(mesh, before);
        }
    }
    #[test]
    fn arcade_wall_vertical_finish_preserves_geometry_and_shadow_material() {
        let wall=model(ModelKind::ArcadeWall);
        let mut mesh=SurfaceMeshData::default();
        wall.append_fitted(&mut mesh,[0.,16.,0.,3.],0.,28.);
        let before=mesh.clone();wall.apply_arcade_wall_finish(&mut mesh,0);
        assert_eq!(mesh.positions,before.positions);
        assert_eq!(mesh.normals,before.normals);
        assert_eq!(mesh.indices,before.indices);
        assert_eq!(mesh.colors,before.colors);
        let mut front_vertical_range=false;
        for (range,kind) in &wall.room_finishes {
            let values=range.clone().filter(|&i|wall.surface.normals[i][2]>0.5)
                .map(|i|mesh.uvs[i][1]-*kind as f32*1000.).collect::<Vec<_>>();
            if !values.is_empty() && values.iter().copied().fold(f32::NEG_INFINITY,f32::max)
                -values.iter().copied().fold(f32::INFINITY,f32::min)>10. {front_vertical_range=true;}
        }
        assert!(front_vertical_range);
        for i in 0..mesh.positions.len() {
            if !wall.room_finishes.iter().any(|(range,_)|range.contains(&i)) {
                assert_eq!(mesh.uvs[i],before.uvs[i]);
            }
        }
    }
    #[test]
    fn arcade_named_black_pigment_is_valid_and_black_screens_keep_their_paint() {
        let mut export: serde_json::Value=serde_json::from_str(
            include_str!("../models/interiors/arcade_machine.mesh.json")).unwrap();
        for primitive in export["primitives"].as_array_mut().unwrap() {
            if matches!(primitive["material"].as_str(),Some("purple" | "screen")) {
                primitive["base_color"]=serde_json::json!([0.,0.,0.,1.]);
            }
        }
        let json=serde_json::to_string(&export).unwrap();
        let arcade=Model::parse(json.as_str()).unwrap();
        let mut mesh=SurfaceMeshData::default();
        arcade.append_fitted(&mut mesh,[0.,16.,0.,12.],0.,21.);
        let before=mesh.clone();arcade.apply_arcade_finish(&mut mesh,0);
        assert_eq!(mesh.positions,before.positions);
        assert_eq!(mesh.normals,before.normals);
        assert_eq!(mesh.indices,before.indices);
        let mut screen=0;
        for i in 0..mesh.colors.len() {
            assert!(mesh.colors[i].iter().all(|c| c.is_finite() && (0.0..=1.0).contains(c)));
            if arcade.arcade_palette.iter().any(|paint| paint.vertices.contains(&i)) {
                assert!(mesh.colors[i][0]>0.5);
            } else {
                assert_eq!(mesh.colors[i],before.colors[i]);
                if before.colors[i][..3]==[0.,0.,0.] {
                    screen+=1;assert_eq!(mesh.uvs[i],[0.,0.]);
                }
            }
        }
        assert!(screen>0);
    }
    #[test]
    fn arcade_counter_palette_is_opt_in_and_leaves_store_counter_paint_intact() {
        let counter=model(ModelKind::ReceptionCounter);
        let mut mesh=SurfaceMeshData::default();
        counter.append_fitted(&mut mesh,[0.,32.,0.,8.],0.,10.);
        let store=mesh.clone();
        counter.apply_room_finish(&mut mesh,0);
        assert_eq!(mesh.colors,store.colors);
        counter.apply_arcade_finish(&mut mesh,0);
        assert_eq!(mesh.positions,store.positions);
        assert_eq!(mesh.normals,store.normals);
        assert_eq!(mesh.indices,store.indices);
        assert!(mesh.colors.iter().zip(&store.colors).any(|(a,b)| a!=b));
        for i in 0..mesh.colors.len() {
            if !counter.arcade_palette.iter().any(|paint| paint.vertices.contains(&i)) {
                assert_eq!(mesh.colors[i],store.colors[i]);
            }
            assert_eq!(mesh.colors[i][3],store.colors[i][3]);
        }
    }
    #[test]
    fn arcade_finish_preserves_geometry_and_never_guesses_unknown_paint() {
        let arcade = model(ModelKind::ArcadeMachine);
        let mut mesh = SurfaceMeshData::default();
        arcade.append_fitted(&mut mesh, [0.,16.,0.,12.], 0.,21.);
        let before = mesh.clone();
        arcade.apply_arcade_finish(&mut mesh, 0);
        assert_eq!(mesh.positions, before.positions);
        assert_eq!(mesh.normals, before.normals);
        assert_eq!(mesh.indices, before.indices);
        assert!(mesh.colors.iter().zip(&before.colors).any(|(a,b)| a != b));
        for i in 0..mesh.colors.len() {
            assert_eq!(mesh.colors[i][3], before.colors[i][3]);
            if !arcade.arcade_palette.iter().any(|paint| paint.vertices.contains(&i)) {
                assert_eq!(mesh.colors[i], before.colors[i]);
            }
        }
        let mut export: serde_json::Value = serde_json::from_str(
            include_str!("../models/interiors/arcade_machine.mesh.json")).unwrap();
        for primitive in export["primitives"].as_array_mut().unwrap() {
            primitive["material"] = serde_json::Value::String("unknown_arcade_paint".into());
        }
        let json = serde_json::to_string(&export).unwrap();
        let unknown = Model::parse(json.as_str()).unwrap();
        let mut unrecognized = SurfaceMeshData::default();
        unknown.append_fitted(&mut unrecognized, [0.,16.,0.,12.], 0.,21.);
        let plain = unrecognized.clone();
        unknown.apply_arcade_finish(&mut unrecognized, 0);
        assert_eq!(unrecognized, plain);
    }
    #[test]
    fn store_seat_finish_uses_named_materials_and_preserves_dimensions() {
        let seat = model(ModelKind::Cushion);
        let mut mesh = SurfaceMeshData::default();
        seat.append_fitted(&mut mesh, [0.,16.,0.,12.], 6.,6.);
        let before = mesh.clone();
        seat.apply_room_palette(&mut mesh, 0);
        seat.apply_room_finish(&mut mesh, 0);
        assert_eq!(mesh.positions, before.positions);
        assert_eq!(mesh.normals, before.normals);
        assert_eq!(mesh.indices, before.indices);
        assert!(mesh.uvs.iter().all(|uv| (82000.0..82100.0).contains(&uv[0]) && (2500.0..2517.0).contains(&uv[1])));
        let mut export: serde_json::Value = serde_json::from_str(
            include_str!("../models/interiors/cushion.mesh.json")).unwrap();
        for primitive in export["primitives"].as_array_mut().unwrap() {
            primitive["material"] = serde_json::Value::String("unknown_upholstery".into());
        }
        let json = serde_json::to_string(&export).unwrap();
        let unknown = Model::parse(json.as_str()).unwrap();
        let mut unrecognized = SurfaceMeshData::default();
        unknown.append_fitted(&mut unrecognized, [0.,16.,0.,12.], 6.,6.);
        let plain = unrecognized.clone();
        unknown.apply_room_palette(&mut unrecognized, 0);
        unknown.apply_room_finish(&mut unrecognized, 0);
        assert_eq!(unrecognized, plain);
    }
    #[test]
    fn store_plant_palette_preserves_support_and_unknown_materials() {
        let plant = model(ModelKind::PottedPlant);
        let mut mesh = SurfaceMeshData::default();
        model(ModelKind::Stool).append_fitted(&mut mesh, [0.,8.,0.,8.], 0.,7.);
        let first = mesh.positions.len();
        plant.append_fitted(&mut mesh, [8.,24.,0.,16.], 0.,23.);
        let before = mesh.clone();
        plant.apply_room_palette(&mut mesh, first);
        assert_eq!(mesh.positions, before.positions);
        assert_eq!(mesh.normals, before.normals);
        assert_eq!(mesh.indices, before.indices);
        assert_eq!(mesh.uvs, before.uvs);
        assert_eq!(&mesh.colors[..first], &before.colors[..first]);
        assert!(mesh.colors[first..].iter().zip(&before.colors[first..]).any(|(a,b)| a != b));
        for i in first..mesh.colors.len() {
            assert_eq!(mesh.colors[i][3], before.colors[i][3]);
            if !plant.room_palette.iter().any(|paint| paint.vertices.contains(&(i-first))) {
                assert_eq!(mesh.colors[i], before.colors[i]);
            }
        }
        let chair = model(ModelKind::Chair);
        let mut unrelated = SurfaceMeshData::default();
        chair.append_fitted(&mut unrelated, [0.,8.,0.,8.], 0.,14.);
        let plain = unrelated.clone();
        chair.apply_room_palette(&mut unrelated, 0);
        assert_eq!(unrelated, plain);
    }
    #[test]
    fn scoped_counter_finish_preserves_geometry_and_untagged_materials() {
        let counter=model(ModelKind::ReceptionCounter);
        let mut mesh=SurfaceMeshData::default();
        model(ModelKind::Stool).append_fitted(&mut mesh,[0.,8.,0.,8.],0.,7.);
        let first=mesh.positions.len();
        counter.append_fitted(&mut mesh,[8.,40.,0.,8.],0.,10.);
        let before=mesh.clone();
        counter.apply_room_finish(&mut mesh,first);
        assert_eq!(mesh.positions,before.positions);
        assert_eq!(mesh.normals,before.normals);
        assert_eq!(mesh.colors,before.colors);
        assert_eq!(mesh.indices,before.indices);
        assert_eq!(&mesh.uvs[..first],&before.uvs[..first]);
        assert!(mesh.uvs[first..].iter().any(|uv| (82000.0..82100.0).contains(&uv[0]) && uv[1]<2000.));
        assert!(mesh.uvs[first..].iter().any(|uv| (82000.0..82100.0).contains(&uv[0]) && uv[1]>=2000.));
        assert!(mesh.uvs[first..].iter().any(|uv| *uv==[0.,0.]));
        let unknown=Model::parse(include_str!("../../../assets/open-models/interiors/chair.mesh.json")).unwrap();
        let mut chair=SurfaceMeshData::default();
        unknown.append_fitted(&mut chair,[0.,8.,0.,8.],0.,14.);
        let plain=chair.clone();
        unknown.apply_room_finish(&mut chair,0);
        assert_eq!(chair,plain);
    }
    #[test]
    fn interior_models_are_finite_materialed_full_volume_meshes() {
        for kind in ModelKind::ALL {
            let m = model(*kind);
            assert!(
                m.surface.indices.len() > 100,
                "{} needs authored detail",
                kind.label()
            );
            assert!(
                m.surface.colors.iter().any(|c| c != &m.surface.colors[0]),
                "{} needs material distinction",
                kind.label()
            );
            assert!(
                m.surface.normals.iter().any(|n| n[1] < -0.25),
                "{} has no closed underside",
                kind.label()
            );
            let mut fitted = SurfaceMeshData::default();
            m.append_fitted(&mut fitted, [-2.0, 3.0, -1.0, 4.0], 0.2, 2.7);
            assert!(fitted.positions.iter().all(|p| p[0] >= -2.001
                && p[0] <= 3.001
                && p[1] >= 0.199
                && p[1] <= 2.901
                && p[2] >= -1.001
                && p[2] <= 4.001));
            assert!(
                fitted
                    .normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.0).abs() < 0.001)
            );
        }
    }
    #[test]
    fn invalid_or_flat_interior_models_are_rejected() {
        assert!(Model::parse(r#"{"primitives":[]}"#).is_err());
        assert!(Model::parse(r#"{"primitives":[{"positions":[0,0,0],"normals":[0,0,0],"indices":[0,0,0],"base_color":[1,1,1,1]}]}"#).is_err());
    }
    #[test]
    fn opted_in_long_tables_turn_without_stretching_or_changing_grounding() {
        let make = |align| {
            Model::parse(
                &serde_json::json!({
                    "preserve_aspect":true, "align_footprint":align,
                    "primitives":[{"positions":[0.,0.,0.,4.,0.,0.,4.,2.,1.,0.,2.,1.],
                        "normals":[0.,0.,1.,0.,0.,1.,0.,0.,1.,0.,0.,1.],
                        "indices":[0,1,2,0,2,3],"base_color":[1.,1.,1.,1.]}]
                })
                .to_string(),
            )
            .unwrap()
        };
        let mut direct = SurfaceMeshData::default();
        make(false).append_fitted(&mut direct, [0., 4., 0., 16.], 0.5, 8.);
        let mut fitted = SurfaceMeshData::default();
        make(true).append_fitted(&mut fitted, [0., 4., 0., 16.], 0.5, 8.);
        let size = |m: &SurfaceMeshData| {
            let min = m
                .positions
                .iter()
                .map(|p| Vec3::from_array(*p))
                .fold(Vec3::splat(f32::INFINITY), Vec3::min);
            let max = m
                .positions
                .iter()
                .map(|p| Vec3::from_array(*p))
                .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
            (min, max - min)
        };
        assert_eq!(
            size(&fitted),
            (Vec3::new(0., 0.5, 0.), Vec3::new(4., 8., 16.))
        );
        assert_eq!(size(&direct).1, Vec3::new(4., 2., 1.));
        assert_eq!(fitted.indices, direct.indices);
        assert!(fitted.normals.iter().all(|n| *n == [1., 0., 0.]));
    }

    #[test]
    fn contained_import_retains_proportions_grounding_and_source_footprint() {
        let mut json: serde_json::Value = crate::model_storage::parse(
            include_str!("../../../assets/open-models/interiors/television.mesh.json"),
        )
        .unwrap();
        json["preserve_aspect"] = true.into();
        let model = Model::parse(&json.to_string()).unwrap();
        let mut mesh = SurfaceMeshData::default();
        model.append_fitted(&mut mesh, [-3., 3., -0.7, 0.7], 0.4, 4.);
        let lo = mesh
            .positions
            .iter()
            .copied()
            .map(Vec3::from_array)
            .fold(Vec3::splat(f32::INFINITY), Vec3::min);
        let hi = mesh
            .positions
            .iter()
            .copied()
            .map(Vec3::from_array)
            .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
        let source_size = Vec3::from_array(model.max) - Vec3::from_array(model.min);
        let size = hi - lo;
        assert!((size.x / size.y - source_size.x / source_size.y).abs() < 1e-5);
        assert!((size.z / size.y - source_size.z / source_size.y).abs() < 1e-5);
        assert!((lo.y - 0.4).abs() < 1e-5);
        assert!((lo.x + hi.x).abs() < 1e-5 && (lo.z + hi.z).abs() < 1e-5);
        assert!(
            lo.x >= -3.0001
                && hi.x <= 3.0001
                && lo.z >= -0.7001
                && hi.z <= 0.7001
                && hi.y <= 4.4001
        );
        assert_eq!(mesh.indices, model.surface.indices);
        assert!(
            mesh.normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 1e-5)
        );
    }
}
