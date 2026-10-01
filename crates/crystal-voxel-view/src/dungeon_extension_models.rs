//! Second, independently editable special-environment kit. No game data.
use super::Model;
use std::sync::OnceLock;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExtensionKind {
    RuinsFrieze,
    LeagueWall,
    PassageWall,
    ChampionDragon,
    ShipBulkhead,
    ShipDoor,
    HarborBollard,
    DockRailing,
    HallOfFameTerminal,
    PuzzleDais,
    HarborFerry,
}
impl ExtensionKind {
    pub(crate) const ALL: [Self; 11] = [
        Self::RuinsFrieze,
        Self::LeagueWall,
        Self::PassageWall,
        Self::ChampionDragon,
        Self::ShipBulkhead,
        Self::ShipDoor,
        Self::HarborBollard,
        Self::DockRailing,
        Self::HallOfFameTerminal,
        Self::PuzzleDais,
        Self::HarborFerry,
    ];
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::RuinsFrieze => "special:ruins-inscribed-frieze",
            Self::LeagueWall => "special:league-wall",
            Self::PassageWall => "special:passage-wall",
            Self::ChampionDragon => "special:champion-dragon",
            Self::ShipBulkhead => "special:ship-bulkhead",
            Self::ShipDoor => "special:ship-door",
            Self::HarborBollard => "special:harbor-bollard",
            Self::DockRailing => "special:dock-railing",
            Self::HallOfFameTerminal => "special:hall-of-fame-terminal",
            Self::PuzzleDais => "special:ruins-puzzle-dais",
            Self::HarborFerry => "special:harbor-ferry",
        }
    }
}
pub(crate) fn extension_model(kind: ExtensionKind) -> &'static Model {
    static MODELS: OnceLock<Vec<Model>> = OnceLock::new();
    &MODELS.get_or_init(|| {
        [
            crate::model_storage::include_model!(
                "models/dungeon_extensions/ruins_frieze.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/dungeon_extensions/league_wall.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/dungeon_extensions/passage_wall.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/dungeon_extensions/champion_dragon.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/dungeon_extensions/ship_bulkhead.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/dungeon_extensions/ship_door.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/dungeon_extensions/harbor_bollard.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/dungeon_extensions/dock_railing.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/dungeon_extensions/hall_of_fame_terminal.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/dungeon_extensions/puzzle_dais.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/dungeon_extensions/harbor_ferry.mesh.json"
            ),
        ]
        .into_iter()
        .map(|s| Model::parse(s).expect("validated original extension mesh"))
        .collect()
    })[kind as usize]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extension_meshes_have_finite_full_volume_geometry() {
        for kind in ExtensionKind::ALL {
            let m = extension_model(kind);
            assert!(m.surface.indices.len() > 60);
            assert!((0..3).all(|a| m.max[a] > m.min[a]));
        }
    }
}
