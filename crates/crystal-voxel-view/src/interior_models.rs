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
    positions: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
    base_color: [f32; 4],
}
#[derive(Deserialize)]
struct Export {
    primitives: Vec<Primitive>,
}
pub(crate) struct Model {
    surface: SurfaceMeshData,
    min: [f32; 3],
    max: [f32; 3],
}
impl Model {
    pub(crate) fn parse(json: &str) -> Result<Self, String> {
        let export: Export = crate::model_storage::parse(json)?;
        let mut surface = SurfaceMeshData::default();
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
        }
        if (0..3).any(|axis| !min[axis].is_finite() || max[axis] <= min[axis]) {
            return Err("empty or flat interior model".into());
        }
        Ok(Self { surface, min, max })
    }
    /// Fit the true 3D asset inside a source-verified physical footprint.
    /// Nonuniform fitting correctly inverse-transforms the normals.
    pub(crate) fn append_fitted(
        &self,
        target: &mut SurfaceMeshData,
        bounds: [f32; 4],
        base_y: f32,
        height: f32,
    ) {
        let [west, east, north, south] = bounds;
        let scale = Vec3::new(
            (east - west) / (self.max[0] - self.min[0]),
            height / (self.max[1] - self.min[1]),
            (south - north) / (self.max[2] - self.min[2]),
        );
        let base = target.positions.len() as u32;
        let light = Vec3::new(-0.35, 0.85, 0.40).normalize();
        for ((p, n), c) in self
            .surface
            .positions
            .iter()
            .zip(&self.surface.normals)
            .zip(&self.surface.colors)
        {
            let p = (Vec3::from_array(*p) - Vec3::from_array(self.min)) * scale
                + Vec3::new(west, base_y, north);
            let normal = (Vec3::from_array(*n) / scale).normalize();
            let shade = 0.64 + 0.36 * normal.dot(light).max(0.0);
            target.positions.push(p.to_array());
            target.normals.push(normal.to_array());
            target.uvs.push([0.0, 0.0]);
            target
                .colors
                .push([c[0] * shade, c[1] * shade, c[2] * shade, c[3]]);
        }
        target
            .indices
            .extend(self.surface.indices.iter().map(|&i| base + i));
    }
}
pub(crate) fn model(kind: ModelKind) -> &'static Model {
    macro_rules! load {
        ($slot:ident, $path:literal) => {{
            static $slot: OnceLock<Model> = OnceLock::new();
            $slot.get_or_init(|| {
                Model::parse(include_str!($path)).expect("validated authored interior asset")
            })
        }};
    }
    match kind {
        ModelKind::BroadcastRack => load!(
            BROADCAST_RACK,
            "../models/interiors/broadcast_rack.mesh.json"
        ),
        ModelKind::TowerReception => load!(
            TOWER_RECEPTION,
            "../models/interiors/tower_reception.mesh.json"
        ),
        ModelKind::Stool => load!(STOOL, "../models/interiors/stool.mesh.json"),
        ModelKind::ArcadeStool => load!(ARCADE_STOOL, "../models/interiors/arcade_stool.mesh.json"),
        ModelKind::Chair => load!(CHAIR, "../models/interiors/chair.mesh.json"),
        ModelKind::LinkSeat => load!(LINK_SEAT, "../models/interiors/link_seat.mesh.json"),
        ModelKind::Cushion => load!(CUSHION, "../models/interiors/cushion.mesh.json"),
        ModelKind::DiningTable => load!(DINING_TABLE, "../models/interiors/dining_table.mesh.json"),
        ModelKind::LowTable => load!(LOW_TABLE, "../models/interiors/low_table.mesh.json"),
        ModelKind::CafeTable => load!(CAFE_TABLE, "../models/interiors/cafe_table.mesh.json"),
        ModelKind::BedRed => load!(BED_RED, "../models/interiors/bed_red.mesh.json"),
        ModelKind::BedBlue => load!(BED_BLUE, "../models/interiors/bed_blue.mesh.json"),
        ModelKind::BedGreen => load!(BED_GREEN, "../models/interiors/bed_green.mesh.json"),
        ModelKind::BedPink => load!(BED_PINK, "../models/interiors/bed_pink.mesh.json"),
        ModelKind::PottedPlant => load!(POTTED_PLANT, "../models/interiors/potted_plant.mesh.json"),
        ModelKind::LongPlanter => load!(LONG_PLANTER, "../models/interiors/long_planter.mesh.json"),
        ModelKind::FlowerStand => load!(FLOWER_STAND, "../models/interiors/flower_stand.mesh.json"),
        ModelKind::Bookcase => load!(BOOKCASE, "../models/interiors/bookcase.mesh.json"),
        ModelKind::LabBookcase => load!(LAB_BOOKCASE, "../models/interiors/lab_bookcase.mesh.json"),
        ModelKind::Cabinet => load!(CABINET, "../models/interiors/cabinet.mesh.json"),
        ModelKind::DrawerCabinet => load!(
            DRAWER_CABINET,
            "../models/interiors/drawer_cabinet.mesh.json"
        ),
        ModelKind::PendulumClock => load!(
            PENDULUM_CLOCK,
            "../models/interiors/pendulum_clock.mesh.json"
        ),
        ModelKind::Computer => load!(COMPUTER, "../models/interiors/computer.mesh.json"),
        ModelKind::LabWorkstation => load!(
            LAB_WORKSTATION,
            "../models/interiors/lab_workstation.mesh.json"
        ),
        ModelKind::Television => load!(TELEVISION, "../models/interiors/television.mesh.json"),
        ModelKind::Radio => load!(RADIO, "../models/interiors/radio.mesh.json"),
        ModelKind::GameConsole => load!(GAME_CONSOLE, "../models/interiors/game_console.mesh.json"),
        ModelKind::Keyboard => load!(KEYBOARD, "../models/interiors/keyboard.mesh.json"),
        ModelKind::Refrigerator => {
            load!(REFRIGERATOR, "../models/interiors/refrigerator.mesh.json")
        }
        ModelKind::RetailRefrigerator => load!(
            RETAIL_REFRIGERATOR,
            "../models/interiors/retail_refrigerator.mesh.json"
        ),
        ModelKind::KitchenStove => {
            load!(KITCHEN_STOVE, "../models/interiors/kitchen_stove.mesh.json")
        }
        ModelKind::KitchenSink => load!(KITCHEN_SINK, "../models/interiors/kitchen_sink.mesh.json"),
        ModelKind::RetailShelf => load!(RETAIL_SHELF, "../models/interiors/retail_shelf.mesh.json"),
        ModelKind::GiftShelf => load!(GIFT_SHELF, "../models/interiors/gift_shelf.mesh.json"),
        ModelKind::ReceptionCounter => load!(
            RECEPTION_COUNTER,
            "../models/interiors/reception_counter.mesh.json"
        ),
        ModelKind::LabCounter => load!(LAB_COUNTER, "../models/interiors/lab_counter.mesh.json"),
        ModelKind::HealingMachine => load!(
            HEALING_MACHINE,
            "../models/interiors/healing_machine.mesh.json"
        ),
        ModelKind::HealingConsole => load!(
            HEALING_CONSOLE,
            "../models/interiors/healing_console.mesh.json"
        ),
        ModelKind::ClinicalPartition => load!(
            CLINICAL_PARTITION,
            "../models/interiors/clinical_partition.mesh.json"
        ),
        ModelKind::TimberPartition => load!(
            TIMBER_PARTITION,
            "../models/interiors/timber_partition.mesh.json"
        ),
        ModelKind::LabRestorationMachine => load!(
            LAB_RESTORATION_MACHINE,
            "../models/interiors/lab_restoration_machine.mesh.json"
        ),
        ModelKind::LabDisplayTable => load!(
            LAB_DISPLAY_TABLE,
            "../models/interiors/lab_display_table.mesh.json"
        ),
        ModelKind::ArcadeMachine => load!(
            ARCADE_MACHINE,
            "../models/interiors/arcade_machine.mesh.json"
        ),
        ModelKind::ArcadePair => load!(ARCADE_PAIR, "../models/interiors/arcade_pair.mesh.json"),
        ModelKind::StationBench => {
            load!(STATION_BENCH, "../models/interiors/station_bench.mesh.json")
        }
        ModelKind::StationTurnstile => load!(
            STATION_TURNSTILE,
            "../models/interiors/station_turnstile.mesh.json"
        ),
        ModelKind::GateTerminal => {
            load!(GATE_TERMINAL, "../models/interiors/gate_terminal.mesh.json")
        }
        ModelKind::BroadcastConsole => load!(
            BROADCAST_CONSOLE,
            "../models/interiors/broadcast_console.mesh.json"
        ),
        ModelKind::VendingMachine => load!(
            VENDING_MACHINE,
            "../models/interiors/vending_machine.mesh.json"
        ),
        ModelKind::WallPanel => load!(WALL_PANEL, "../models/interiors/wall_panel.mesh.json"),
        ModelKind::WindowWall => load!(WINDOW_WALL, "../models/interiors/window_wall.mesh.json"),
        ModelKind::GateDoorFrame => load!(
            GATE_DOOR_FRAME,
            "../models/interiors/gate_door_frame.mesh.json"
        ),
        ModelKind::RadioWall => load!(RADIO_WALL, "../models/interiors/radio_wall.mesh.json"),
        ModelKind::OpenBook => load!(OPEN_BOOK, "../models/interiors/open_book.mesh.json"),
        ModelKind::Memorial => load!(MEMORIAL, "../models/interiors/memorial.mesh.json"),
        ModelKind::BedFeathery => load!(BED_FEATHERY, "../models/interiors/bed_feathery.mesh.json"),
        ModelKind::BedPolkadot => load!(BED_POLKADOT, "../models/interiors/bed_polkadot.mesh.json"),
        ModelKind::BedPikachu => load!(BED_PIKACHU, "../models/interiors/bed_pikachu.mesh.json"),
        ModelKind::PlantMagna => load!(PLANT_MAGNA, "../models/interiors/plant_magna.mesh.json"),
        ModelKind::PlantTropic => load!(PLANT_TROPIC, "../models/interiors/plant_tropic.mesh.json"),
        ModelKind::PlantJumbo => load!(PLANT_JUMBO, "../models/interiors/plant_jumbo.mesh.json"),
        ModelKind::WallDomestic => {
            load!(WALL_DOMESTIC, "../models/interiors/wall_domestic.mesh.json")
        }
        ModelKind::WallTraditional => load!(
            WALL_TRADITIONAL,
            "../models/interiors/wall_traditional.mesh.json"
        ),
        ModelKind::WallClinical => {
            load!(WALL_CLINICAL, "../models/interiors/wall_clinical.mesh.json")
        }
        ModelKind::WallAcoustic => {
            load!(WALL_ACOUSTIC, "../models/interiors/wall_acoustic.mesh.json")
        }
        ModelKind::CarpetCloth => load!(CARPET_CLOTH, "../models/interiors/carpet_cloth.mesh.json"),
        ModelKind::PictureFrame => {
            load!(PICTURE_FRAME, "../models/interiors/picture_frame.mesh.json")
        }
        ModelKind::StairFlight => load!(STAIR_FLIGHT, "../models/interiors/stair_flight.mesh.json"),
        ModelKind::OfficePhone => load!(OFFICE_PHONE, "../models/interiors/office_phone.mesh.json"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
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
}
