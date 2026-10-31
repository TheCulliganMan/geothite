//! Stable identities for independently owned Minnesota daycare residents.

use crate::state::{DayCareState, GameState, OverworldMemory};

/// Save keys are independent of exterior coordinates and runtime map numbers.
pub const MANKATO_DAY_CARE: &str = "minnesota-mankato";
pub const BEMIDJI_DAY_CARE: &str = "minnesota-bemidji";

const EMPTY: DayCareState = DayCareState {
    man: crate::state::DayCareResidentState {
        pokemon: None,
        active: false,
    },
    lady: crate::state::DayCareResidentState {
        pokemon: None,
        active: false,
    },
    compatibility_score: 0,
    egg_present: false,
    egg: None,
    steps_until_next_egg: 0,
};

/// Resolves a registered daycare interior to its persistent custody identity.
pub fn location_id(map: &str) -> Option<&'static str> {
    match map {
        "MnMankatoDayCare" => Some(MANKATO_DAY_CARE),
        "MnBemidjiDayCare" => Some(BEMIDJI_DAY_CARE),
        _ => None,
    }
}

impl GameState {
    /// The current site's identity, or `None` for Crystal's original daycare.
    pub fn current_day_care_id(&self) -> Option<&'static str> {
        match &self.overworld {
            OverworldMemory::Active { map_name, .. } => location_id(map_name),
            OverworldMemory::Inactive => None,
        }
    }

    /// Reads only this location's residents and egg. Unvisited sites are empty.
    pub fn current_day_care(&self) -> &DayCareState {
        match self.current_day_care_id() {
            Some(id) => self.day_care_locations.get(id).unwrap_or(&EMPTY),
            None => &self.day_care,
        }
    }

    /// Mutates only this location's custody; never transfers between sites.
    pub fn current_day_care_mut(&mut self) -> &mut DayCareState {
        match self.current_day_care_id() {
            Some(id) => self.day_care_locations.entry(id.to_owned()).or_default(),
            None => &mut self.day_care,
        }
    }
}

pub(crate) fn omit_locations_for_legacy_save(
    _: &std::collections::BTreeMap<String, DayCareState>,
) -> bool {
    crate::save::pre_regional_day_care_wire()
}

pub(crate) fn deserialize_locations<'de, D>(
    deserializer: D,
) -> Result<std::collections::BTreeMap<String, DayCareState>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    if crate::save::pre_regional_day_care_wire() && !deserializer.is_human_readable() {
        Ok(std::collections::BTreeMap::new())
    } else {
        serde::Deserialize::deserialize(deserializer)
    }
}
