//! Operative fields of the reviewed, detached full-state cohesion contracts.
use std::collections::BTreeMap;

use serde::Deserialize;

use crate::authored_route_design::{Facing, Sign};

#[derive(Deserialize)]
pub(crate) struct Plan {
    pub schema_version: u16,
    pub id: String,
    pub base_pack_sha256: String,
    pub base_content_hash: String,
    pub map_count_before: usize,
    pub registry: Registry,
    pub new_maps: Vec<NewMap>,
    pub native_preimages: Vec<Preimage>,
    pub river_module_preservation_preimages: Vec<Preimage>,
    pub native_port_patches: String,
    pub native_port_patches_sha256: String,
    pub new_reciprocal_links: Vec<Link>,
    pub existing_connection_updates: Vec<OffsetUpdate>,
    pub guard_proposals: Vec<Guard>,
    pub landmark_bindings: Vec<Landmark>,
    pub encounter_bindings_proposed: Vec<EncounterBinding>,
    pub proposed_origins_metatiles: BTreeMap<String, [i32; 2]>,
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
    pub dimensions_metatiles: [u16; 2],
    pub layout_file: String,
    pub layout_sha256: String,
    pub dossier_file: String,
    pub dossier_sha256: String,
    pub tileset: String,
    pub environment: String,
    pub music: String,
}

#[derive(Deserialize)]
pub(crate) struct Preimage {
    pub map: String,
    pub module_sha256: String,
}

#[derive(Deserialize)]
pub(crate) struct Link {
    pub from_map: String,
    pub direction: String,
    pub to_map: String,
    pub offset: i32,
    pub reverse_offset: i32,
    pub from_core: [[u16; 2]; 2],
    pub to_core: [[u16; 2]; 2],
}

#[derive(Deserialize)]
pub(crate) struct OffsetUpdate {
    pub map: String,
    pub side: String,
    pub target: String,
    pub old_offset: i32,
    pub new_offset: i32,
}

#[derive(Deserialize)]
pub(crate) struct Guard {
    pub map: String,
    pub id: String,
    pub side: String,
    pub bodies: [[u16; 2]; 2],
    pub inside: [[u16; 2]; 2],
    pub outside: [[u16; 2]; 2],
    pub flags: Vec<String>,
    #[serde(default)]
    pub event: String,
    pub actor_ids: [String; 2],
    pub sprite: String,
    pub facing: String,
    pub outbound_only: bool,
    pub inbound_free: bool,
    pub denial_text: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct Landmark {
    pub map: String,
    pub reference_native_map: String,
    pub landmark_constant: String,
    pub existing_landmark_id: u16,
    pub add_landmark_row: bool,
}

#[derive(Deserialize)]
pub(crate) struct EncounterBinding {
    pub map: String,
    pub native_reference_map: String,
}

#[derive(Deserialize)]
pub(crate) struct Patches {
    pub parent_pack_sha256: String,
    pub patches: Vec<Patch>,
}

#[derive(Deserialize)]
pub(crate) struct Patch {
    pub map: String,
    pub native_module_sha256: String,
    pub dimensions_metatiles: Option<[u16; 2]>,
    #[serde(default)]
    pub changes: Vec<Cell>,
    pub kind: Option<String>,
    pub before_dimensions_metatiles: Option<[u16; 2]>,
    pub after_dimensions_metatiles: Option<[u16; 2]>,
    pub removed_bounds_metatiles: Option<[u16; 4]>,
}

#[derive(Deserialize)]
pub(crate) struct Cell {
    pub metatile: [u16; 2],
    pub before: u16,
    pub after: u16,
}

#[derive(Deserialize)]
pub(crate) struct Dossier {
    pub id: String,
    pub ports: Vec<Port>,
    pub actors: Vec<Actor>,
    pub signs: Vec<Sign>,
}

#[derive(Deserialize)]
pub(crate) struct Port {
    pub side: String,
    pub core: [[u16; 2]; 2],
}

#[derive(Deserialize)]
pub(crate) struct Actor {
    pub id: String,
    pub tile: [u16; 2],
    pub sprite: String,
    pub facing: Facing,
    pub role: String,
    pub trainer_name: String,
    pub trainer_class: String,
    pub party: Vec<crate::authored_route_design::PartyMember>,
    pub event_flag: String,
    pub text: Vec<String>,
}
