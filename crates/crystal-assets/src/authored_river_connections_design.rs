//! Source-only contracts for explicit additive river maps and bridge apertures.
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Manifest {
    pub schema_version: u16,
    pub id: String,
    pub base_pack_sha256: String,
    pub base_content_hash: Option<String>,
    pub registry: Registry,
    pub new_maps: Vec<NewMap>,
    #[serde(default)]
    pub terrain_revisions: Vec<TerrainRevision>,
    #[serde(default)]
    pub existing_connection_additions: Vec<ConnectionAddition>,
    pub bridge_inventory: String,
    #[serde(default)]
    pub objects: Vec<Actor>,
    #[serde(default)]
    pub backgrounds: Vec<serde_json::Value>,
    #[serde(default)]
    pub encounter_contracts: Vec<Encounters>,
}

#[derive(Deserialize)]
pub(crate) struct Registry {
    pub group_id: u16,
    pub group_name: String,
    pub map_group_constant: String,
}

#[derive(Deserialize)]
pub(crate) struct NewMap {
    pub id: String,
    pub map_id: u16,
    pub map_constant: String,
    pub layout: String,
    pub layout_sha256: String,
    pub source_origin_metatiles: [u16; 2],
    pub dimensions_metatiles: [u16; 2],
    pub world_origin_metatiles: [i32; 2],
    pub tileset: String,
    pub environment: String,
    pub music: String,
    pub connections: Vec<Connection>,
    pub source_blocks_sha256: String,
}

#[derive(Deserialize)]
pub(crate) struct Connection {
    pub direction: String,
    pub target_map: String,
    pub offset: i32,
}

#[derive(Deserialize)]
pub(crate) struct ConnectionAddition {
    pub map: String,
    pub direction: String,
    pub target_map: String,
    pub offset: i32,
}

#[derive(Deserialize)]
pub(crate) struct TerrainRevision {
    pub map: String,
    pub native_module_sha256: String,
    pub dimensions_metatiles: [u16; 2],
    pub changes: Vec<CellChange>,
    pub anchor_preimages: Anchors,
}

#[derive(Deserialize)]
pub(crate) struct CellChange {
    pub metatile: [u16; 2],
    pub before: u16,
    pub after: u16,
}

#[derive(Deserialize)]
pub(crate) struct Anchors {
    pub objects: Vec<[u16; 2]>,
    pub backgrounds: Vec<[u16; 2]>,
    pub coordinates: Vec<[u16; 2]>,
    pub warps: Vec<[u16; 2]>,
}

#[derive(Deserialize)]
pub(crate) struct BridgeInventory {
    pub schema_version: u16,
    pub base_pack_sha256: String,
    pub bridges: Vec<Bridge>,
}

#[derive(Deserialize)]
pub(crate) struct Bridge {
    pub id: String,
    pub map: String,
    pub tileset: String,
    pub native_module_sha256: String,
    pub metatile_bounds_exclusive: [u16; 4],
    /// Direction of water flow below the walking deck.
    pub axis: String,
    pub water_mouths_core: Vec<[u16; 2]>,
    pub aperture_width_core: u16,
    pub metatiles: Vec<BridgeCell>,
    pub existing_foot_collision_must_remain_exact: bool,
}

#[derive(Deserialize)]
pub(crate) struct BridgeCell {
    pub metatile: [u16; 2],
    pub block_decimal: u16,
    pub block_hex: String,
    pub foot_collision: Vec<String>,
    pub proposed_surf_underpass_collision: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct Actor {
    pub id: String,
    pub map: String,
    pub core: [u16; 2],
    pub sprite: String,
    pub role: String,
    pub facing: String,
    #[serde(default)]
    pub text: Vec<String>,
    pub trainer_class: Option<String>,
    pub trainer_name: Option<String>,
    #[serde(default)]
    pub party: Vec<PartyMember>,
    pub event_flag: Option<String>,
    pub item: Option<String>,
    pub quantity: Option<u16>,
    pub native_special: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct PartyMember {
    pub species: String,
    pub level: u8,
}

#[derive(Deserialize)]
pub(crate) struct Encounters {
    pub map: Option<String>,
    #[serde(default)]
    pub maps: Vec<String>,
    pub terrain: String,
    pub slots: Vec<EncounterSlot>,
}

#[derive(Deserialize)]
pub(crate) struct EncounterSlot {
    pub species: String,
    pub level: u8,
    pub weight: u8,
}
