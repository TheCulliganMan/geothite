//! Original editable special-room kit; geometry contains no game artwork.
use super::Model;
use std::sync::OnceLock;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RoomAsset {
    BicycleSide,
    BicycleFront,
    BicycleDiagonal,
    BikeServiceRack,
    MobileBattleTerminal,
    MobileTradeTerminal,
    LinkBattleConsole,
    LinkTradeConsole,
    ShrineDragonRail,
    ShrineDragonMask,
    ShrinePaperLantern,
    TowerEmblemPanel,
    TowerRoundFixture,
    ElevatorControls,
    PrizeCounter,
    RoofAccessHut,
    RoofPlanter,
    GymStonePartition,
    FacilityInstrumentBank,
    LinkRoomReceiver,
    FacilityInstrumentPair,
    RoofBinoculars,
    LinkRoundStool,
}
impl RoomAsset {
    pub(crate) const ALL: [Self; 23] = [
        Self::BicycleSide,
        Self::BicycleFront,
        Self::BicycleDiagonal,
        Self::BikeServiceRack,
        Self::MobileBattleTerminal,
        Self::MobileTradeTerminal,
        Self::LinkBattleConsole,
        Self::LinkTradeConsole,
        Self::ShrineDragonRail,
        Self::ShrineDragonMask,
        Self::ShrinePaperLantern,
        Self::TowerEmblemPanel,
        Self::TowerRoundFixture,
        Self::ElevatorControls,
        Self::PrizeCounter,
        Self::RoofAccessHut,
        Self::RoofPlanter,
        Self::GymStonePartition,
        Self::FacilityInstrumentBank,
        Self::LinkRoomReceiver,
        Self::FacilityInstrumentPair,
        Self::RoofBinoculars,
        Self::LinkRoundStool,
    ];
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::BicycleSide => "special-room:bicycle-side",
            Self::BicycleFront => "special-room:bicycle-front",
            Self::BicycleDiagonal => "special-room:bicycle-diagonal",
            Self::BikeServiceRack => "special-room:bike-service-rack",
            Self::MobileBattleTerminal => "special-room:mobile-battle-terminal",
            Self::MobileTradeTerminal => "special-room:mobile-trade-terminal",
            Self::LinkBattleConsole => "special-room:link-battle-console",
            Self::LinkTradeConsole => "special-room:link-trade-console",
            Self::ShrineDragonRail => "special-room:shrine-dragon-rail",
            Self::ShrineDragonMask => "special-room:shrine-dragon-mask",
            Self::ShrinePaperLantern => "special-room:shrine-paper-lantern",
            Self::TowerEmblemPanel => "special-room:tower-emblem-panel",
            Self::TowerRoundFixture => "special-room:tower-round-fixture",
            Self::ElevatorControls => "special-room:elevator-controls",
            Self::PrizeCounter => "special-room:prize-counter",
            Self::RoofAccessHut => "special-room:roof-access-hut",
            Self::RoofPlanter => "special-room:roof-planter",
            Self::GymStonePartition => "special-room:gym-stone-partition",
            Self::FacilityInstrumentBank => "special-room:facility-instrument-bank",
            Self::LinkRoomReceiver => "special-room:link-room-receiver",
            Self::FacilityInstrumentPair => "special-room:facility-instrument-pair",
            Self::RoofBinoculars => "special-room:roof-binoculars",
            Self::LinkRoundStool => "special-room:link-round-stool",
        }
    }
}
pub(crate) fn room_model(kind: RoomAsset) -> &'static Model {
    static MODELS: OnceLock<Vec<Model>> = OnceLock::new();
    &MODELS.get_or_init(|| {
        [
            crate::model_storage::include_model!("models/special_rooms/bicycle_side.mesh.json"),
            crate::model_storage::include_model!("models/special_rooms/bicycle_front.mesh.json"),
            crate::model_storage::include_model!(
                "models/special_rooms/bicycle_diagonal.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/special_rooms/bike_service_rack.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/special_rooms/mobile_battle_terminal.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/special_rooms/mobile_trade_terminal.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/special_rooms/link_battle_console.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/special_rooms/link_trade_console.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/special_rooms/shrine_dragon_rail.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/special_rooms/shrine_dragon_mask.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/special_rooms/shrine_paper_lantern.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/special_rooms/tower_emblem_panel.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/special_rooms/tower_round_fixture.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/special_rooms/elevator_controls.mesh.json"
            ),
            crate::model_storage::include_model!("models/special_rooms/prize_counter.mesh.json"),
            crate::model_storage::include_model!(
                "models/special_rooms/roof_access_hut.mesh.json"
            ),
            crate::model_storage::include_model!("models/special_rooms/roof_planter.mesh.json"),
            crate::model_storage::include_model!(
                "models/special_rooms/gym_stone_partition.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/special_rooms/facility_instrument_bank.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/special_rooms/link_room_receiver.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/special_rooms/facility_instrument_pair.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/special_rooms/roof_binoculars.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/special_rooms/link_round_stool.mesh.json"
            ),
        ]
        .into_iter()
        .map(|s| Model::parse(s).expect("validated original special-room mesh"))
        .collect()
    })[kind as usize]
}
impl Model {
    /// Keep the authored volume while resolving each semantic material against
    /// a texel in the current source drawing and its active palette.
    pub(crate) fn append_source_sampled(
        &self,
        target: &mut crate::mesh::SurfaceMeshData,
        bounds: [f32; 4],
        base_height: f32,
        height: f32,
        source_uv: impl Fn([f32; 2]) -> [f32; 2],
    ) {
        let first = target.positions.len();
        self.append(target, bounds, base_height, height);
        for (uv, color) in target.uvs[first..]
            .iter_mut()
            .zip(&mut target.colors[first..])
        {
            *uv = source_uv(*uv);
            *color = [1.; 4];
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_sampled_room_assets_preserve_geometry_and_resolve_every_material_live() {
        for (kind, height) in [
            (RoomAsset::RoofBinoculars, 16.),
            (RoomAsset::FacilityInstrumentPair, 16.),
            (RoomAsset::FacilityInstrumentBank, 32.),
            (RoomAsset::LinkRoundStool, 16.),
        ] {
            let model = room_model(kind);
            assert!(model
                .surface
                .uvs
                .iter()
                .all(|uv| uv[0] >= 0. && uv[0] < 16. && uv[1] >= 0. && uv[1] < height));
            assert!(model.surface.uvs.iter().any(|uv| *uv != [0., 0.]));
            assert!(model.surface.indices.len() / 3 < 1000, "{kind:?}");
            let mut solid = crate::mesh::SurfaceMeshData::default();
            let mut sampled = crate::mesh::SurfaceMeshData::default();
            model.append(&mut solid, [10., 26., 20., 36.], 2., 12.);
            model.append_source_sampled(&mut sampled, [10., 26., 20., 36.], 2., 12., |uv| {
                [uv[0] / 16., uv[1] / height]
            });
            assert_eq!(solid.positions, sampled.positions);
            assert_eq!(solid.normals, sampled.normals);
            assert_eq!(solid.indices, sampled.indices);
            assert!(sampled.colors.iter().all(|color| *color == [1.; 4]));
            assert!(sampled
                .uvs
                .iter()
                .all(|uv| uv.iter().all(|v| (0. ..1.).contains(v))));
            let previous_uvs = sampled.uvs.clone();
            model
                .append_source_sampled(&mut sampled, [40., 56., 50., 66.], 2., 12., |_| [0.9, 0.8]);
            assert_eq!(sampled.uvs[..previous_uvs.len()], previous_uvs);
            assert!(sampled.uvs[previous_uvs.len()..]
                .iter()
                .all(|uv| *uv == [0.9, 0.8]));
        }
    }
    #[test]
    fn special_room_original_meshes_have_volume_and_normalized_finite_geometry() {
        for a in RoomAsset::ALL {
            let m = room_model(a);
            assert!(m.surface.indices.len() > 60);
            assert!((0..3).all(|i| m.max[i] > m.min[i]));
            assert!(m.surface.positions.iter().flatten().all(|v| v.is_finite()));
            assert!(m
                .surface
                .normals
                .iter()
                .all(|n| (bevy::prelude::Vec3::from_array(*n).length() - 1.).abs() < 0.001));
        }
    }
}
