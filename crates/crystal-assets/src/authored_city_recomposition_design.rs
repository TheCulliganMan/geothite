//! Exact anchor contracts for recomposing an already installed city.

use serde::Deserialize;

use crate::authored_city_expansion_design::{Anchor, Connection, Expansion, Spawn};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Recomposition {
    pub schema_version: u16,
    pub city: String,
    pub native_module_sha256: String,
    pub old_dimensions_metatiles: [u16; 2],
    pub dimensions_metatiles: [u16; 2],
    pub arrival_core: [u16; 2],
    pub anchor_overrides: Vec<Anchor>,
    pub connections: Vec<Connection>,
    pub spawn_overrides: Vec<Spawn>,
    #[serde(default)]
    pub waterfall_block: Option<u16>,
    #[serde(default)]
    pub north_water_checkpoint: bool,
    #[serde(default)]
    pub water_seams: Vec<WaterSeam>,
    #[serde(default)]
    pub neighbor_terrain: Vec<NeighborTerrain>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WaterSeam {
    pub direction: String,
    pub target: String,
    pub town_edge: Vec<[u16; 2]>,
    pub route_edge: Vec<[u16; 2]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NeighborTerrain {
    pub map: String,
    pub native_module_sha256: String,
    pub changes: Vec<BlockChange>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BlockChange {
    pub metatile: [u16; 2],
    pub before: u16,
    pub after: u16,
}

impl Recomposition {
    /// Reuse the existing indexed relocation and native geometry validators.
    /// This adds no content and cannot enter the old expansion publisher.
    pub fn into_relocation(self) -> Expansion {
        Expansion {
            schema_version: self.schema_version,
            city: self.city,
            native_module_sha256: self.native_module_sha256,
            old_dimensions_metatiles: self.old_dimensions_metatiles,
            dimensions_metatiles: self.dimensions_metatiles,
            translation_core: [0, 0],
            arrival_core: self.arrival_core,
            clinic_constant: String::new(),
            clinic_recovery_core: None,
            default_mart: String::new(),
            anchor_overrides: self.anchor_overrides,
            connections: self.connections,
            spawn_overrides: self.spawn_overrides,
            residents: Vec::new(),
            signs: Vec::new(),
            doors: Vec::new(),
            rooms: Vec::new(),
            event_flags: Vec::new(),
            programs: Default::default(),
            trainers: Vec::new(),
            items: Vec::new(),
            text_revisions: Vec::new(),
        }
    }
}
