//! Pack-bound preserve geography, weighted habitats and saved-origin validation.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::state::{GameState, OverworldMemory};
use crystal_core::systems::safari::{self, SafariEncounter, SafariPhase, SafariReturn};
use crystal_core::world::map::TilePosition;
use serde::{Deserialize, Serialize};

use crate::{GameDataSet, MapModule};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum SafariSurface {
    Grass,
    Water,
}

/// A native map's placement in the hand-authored preserve, or an indoor stop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafariMap {
    pub origin_core: Option<[u16; 2]>,
}

/// Existing free walking approach and fenced court, before the permit guards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafariGateCourt {
    pub map_name: String,
    pub free_bounds_core: [[u16; 2]; 2],
    pub approach_map: String,
    pub guard_identifiers: Vec<String>,
}

impl SafariGateCourt {
    pub(crate) fn contains(&self, name: &str, tile: TilePosition) -> bool {
        let [min, max] = self.free_bounds_core;
        name == self.map_name
            && tile.x >= 0
            && tile.y >= 0
            && (min[0]..max[0]).contains(&(tile.x as u16))
            && (min[1]..max[1]).contains(&(tile.y as u16))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafariWildSlot {
    pub species: String,
    pub level: u8,
    pub weight: u8,
    pub flee_threshold: u8,
}

/// Deliberate geographic habitat; bounds use half-open global gameplay tiles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafariHabitat {
    pub id: String,
    pub bounds_core: [[u16; 2]; 2],
    pub grass_rate: u8,
    pub water_rate: u8,
    pub grass: Vec<SafariWildSlot>,
    pub water: Vec<SafariWildSlot>,
}

impl SafariHabitat {
    pub(crate) fn table(&self, surface: SafariSurface) -> &[SafariWildSlot] {
        match surface {
            SafariSurface::Grass => &self.grass,
            SafariSurface::Water => &self.water,
        }
    }

    pub(crate) fn contains(&self, at: [u16; 2]) -> bool {
        let [min, max] = self.bounds_core;
        (min[0]..max[0]).contains(&at[0]) && (min[1]..max[1]).contains(&at[1])
    }
}

/// Optional native capture expedition. Fixed30/500/500 rules live in core;
/// configuration binds those rules to real maps, scripts, safe tiles and fauna.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafariZone {
    pub version: u8,
    pub id: String,
    pub admission_map: String,
    pub admission_script: String,
    #[serde(default)]
    pub retire_scripts: Vec<String>,
    pub admission_actor_core: [u16; 2],
    pub entry: SafariReturn,
    pub return_to: SafariReturn,
    pub gate_court: SafariGateCourt,
    pub maps: BTreeMap<String, SafariMap>,
    pub habitats: Vec<SafariHabitat>,
    #[serde(
        default = "default_fishing_flee_threshold",
        skip_serializing_if = "is_default_fishing_flee_threshold"
    )]
    pub fishing_flee_threshold: u8,
}

fn default_fishing_flee_threshold() -> u8 {
    64
}
fn is_default_fishing_flee_threshold(value: &u8) -> bool {
    *value == default_fishing_flee_threshold()
}

impl SafariZone {
    /// Validate actual native references, full habitat coverage, hundred-weight
    /// tables and legal evolution levels before pack/runtime publication.
    ///
    /// # Errors
    /// Rejects missing or unsafe maps/anchors, overlapping geography, ordinary
    /// encounter substitutes, empty/invalid tables, absent species or underlevel
    /// evolutions. Does not certify admission/menu/controller implementation.
    pub fn validate(&self, data: &GameDataSet) -> Result<()> {
        crate::safari_roster::validate_safari_roster(data, self)?;
        ensure!(self.version == 1, "unsupported Safari config version");
        ensure!(
            self.fishing_flee_threshold != 0,
            "Safari fishing requires a nonzero flee threshold"
        );
        token(&self.id)?;
        token(&self.admission_script)?;
        ensure!(
            !self.maps.is_empty() && !self.habitats.is_empty(),
            "Safari requires maps and habitats"
        );
        ensure!(
            !self.maps.contains_key(&self.admission_map)
                && !self.maps.contains_key(&self.return_to.map_name),
            "Safari admission and return must remain in the free hub"
        );
        ensure!(
            self.admission_map == self.return_to.map_name,
            "Safari returns to its free admission hub"
        );
        ensure!(
            self.maps
                .get(&self.entry.map_name)
                .is_some_and(|map| map.origin_core.is_some()),
            "Safari entry must be an exterior preserve map"
        );
        validate_arrival(data, &self.entry)?;
        validate_arrival(data, &self.return_to)?;
        ensure!(
            self.maps.contains_key(&self.gate_court.map_name)
                && !self.maps.contains_key(&self.gate_court.approach_map),
            "gate court must retain its free outside walking approach"
        );
        ensure!(
            self.gate_court
                .contains(&self.entry.map_name, self.entry.tile),
            "Safari entry must land safely inside the free gate court"
        );
        let court = data
            .maps
            .get(&self.gate_court.map_name)
            .context("Safari court map missing")?;
        ensure!(
            !self.gate_court.guard_identifiers.is_empty(),
            "Safari court requires actual permit guards"
        );
        let mut guards = std::collections::BTreeSet::new();
        for guard in &self.gate_court.guard_identifiers {
            ensure!(
                guards.insert(guard)
                    && court
                        .objects
                        .iter()
                        .any(|actor| actor.object_identifier.as_ref() == Some(guard)),
                "missing or duplicate Safari permit guard {guard}"
            );
        }
        let hub = data
            .maps
            .get(&self.admission_map)
            .context("Safari free admission map missing")?;
        let [x, y] = self.admission_actor_core;
        ensure!(
            hub.objects
                .iter()
                .any(|actor| actor.x == x && actor.y == y && actor.script == self.admission_script),
            "Safari admission must bind the actual ranger actor/script"
        );
        let mut retire_scripts = std::collections::BTreeSet::new();
        for script in &self.retire_scripts {
            token(script)?;
            ensure!(
                script != &self.admission_script && retire_scripts.insert(script),
                "duplicate or admission script used as a Safari return ranger"
            );
            ensure!(
                self.maps.keys().any(|name| data
                    .maps
                    .get(name)
                    .is_some_and(|map| map.objects.iter().any(|actor| &actor.script == script))),
                "Safari return ranger script has no preserve actor: {script}"
            );
        }
        let mut ids = std::collections::BTreeSet::new();
        for habitat in &self.habitats {
            token(&habitat.id)?;
            ensure!(
                ids.insert(&habitat.id),
                "duplicate Safari habitat {}",
                habitat.id
            );
            let [min, max] = habitat.bounds_core;
            ensure!(
                min[0] < max[0] && min[1] < max[1],
                "empty Safari habitat {}",
                habitat.id
            );
            for surface in [SafariSurface::Grass, SafariSurface::Water] {
                let slots = habitat.table(surface);
                ensure!(
                    !slots.is_empty()
                        && slots.len() <= 100
                        && slots.iter().map(|slot| u16::from(slot.weight)).sum::<u16>() == 100,
                    "Safari {} {surface:?} needs a hundred-weight table",
                    habitat.id
                );
                let mut species = std::collections::BTreeSet::new();
                for slot in slots {
                    ensure!(
                        species.insert(&slot.species),
                        "duplicate species in Safari {} {surface:?}",
                        habitat.id
                    );
                    ensure!(
                        slot.weight != 0
                            && slot.flee_threshold != 0
                            && (1..=100).contains(&slot.level),
                        "invalid Safari wildlife slot"
                    );
                    data.pokemon
                        .get(&slot.species)
                        .with_context(|| format!("Safari species {} missing", slot.species))?
                        .validate_compiled_shape()
                        .map_err(anyhow::Error::msg)?;
                    let predecessors = data
                        .evolutions
                        .0
                        .values()
                        .flatten()
                        .filter(|evolution| evolution.species == slot.species)
                        .collect::<Vec<_>>();
                    if !predecessors.is_empty()
                        && predecessors
                            .iter()
                            .all(|evolution| evolution.method == "LEVEL")
                    {
                        let minimum = predecessors
                            .iter()
                            .filter_map(|evolution| evolution.level)
                            .min()
                            .context("Safari evolution level missing")?;
                        ensure!(
                            slot.level >= minimum,
                            "Safari {} level{} is below evolution level{}",
                            slot.species,
                            slot.level,
                            minimum
                        );
                    }
                }
            }
        }
        for (name, placement) in &self.maps {
            data.safari_caught_location(name)?;
            let map = data
                .maps
                .get(name)
                .with_context(|| format!("Safari map {name} missing"))?;
            ensure!(
                !data.wild_encounters.contains_key(name)
                    && !data.field_encounters.contains_key(name),
                "Safari {name} cannot also dispatch ordinary encounters"
            );
            if let Some(origin) = placement.origin_core {
                let (width, height) = native_bounds(map)?;
                for y in 0..height {
                    for x in 0..width {
                        let global = [
                            origin[0].checked_add(x).context("Safari x overflow")?,
                            origin[1].checked_add(y).context("Safari y overflow")?,
                        ];
                        ensure!(
                            self.habitats
                                .iter()
                                .filter(|habitat| habitat.contains(global))
                                .count()
                                == 1,
                            "Safari {name} core({x},{y}) must have exactly one habitat"
                        );
                    }
                }
            }
            // Every declared native connection must stay inside the preserve.
            // Admission/return are explicit transactions, not open water seams.
            for edge in &map.attributes.connections {
                let court_approach = *name == self.gate_court.map_name
                    && edge.target_map == self.gate_court.approach_map;
                ensure!(
                    self.maps.contains_key(&edge.target_map) || court_approach,
                    "Safari {name} has outward seam to {}",
                    edge.target_map
                );
                let target = data
                    .maps
                    .get(&edge.target_map)
                    .context("Safari connection target map missing")?;
                let reverse = match edge.direction.as_str() {
                    "north" => "south",
                    "south" => "north",
                    "east" => "west",
                    "west" => "east",
                    _ => anyhow::bail!("invalid Safari seam direction"),
                };
                let reverse_offset = edge
                    .offset
                    .checked_neg()
                    .context("Safari seam offset overflow")?;
                ensure!(
                    target
                        .attributes
                        .connections
                        .iter()
                        .any(|back| back.target_map == *name
                            && back.direction == reverse
                            && back.offset == reverse_offset),
                    "Safari {name} seam must be reciprocal"
                );
            }
            for warp in &map.events.warps {
                let target = if data.maps.contains_key(&warp.target_map) {
                    warp.target_map.clone()
                } else {
                    data.map_name_for_constant(&warp.target_map_constant)?
                };
                ensure!(
                    self.maps.contains_key(&target),
                    "Safari {name} has outward warp to {target}"
                );
            }
        }
        Ok(())
    }

    /// Find the actual geographic habitat at a native local tile. Interior
    /// hospitality stops are included in the permit but have no wild habitat.
    ///
    /// # Errors
    /// Rejects absent/out-of-bounds maps or uncovered/overlapping terrain.
    pub fn habitat_at<'a>(
        &'a self,
        data: &GameDataSet,
        map_name: &str,
        tile: TilePosition,
    ) -> Result<Option<&'a SafariHabitat>> {
        let map = data
            .maps
            .get(map_name)
            .context("Safari position map missing")?;
        let (width, height) = native_bounds(map)?;
        ensure!(
            tile.x >= 0 && tile.y >= 0 && tile.x < width as i16 && tile.y < height as i16,
            "Safari position outside native map"
        );
        let placement = self
            .maps
            .get(map_name)
            .context("position is outside preserve")?;
        if self.gate_court.contains(map_name, tile) {
            return Ok(None);
        }
        let Some(origin) = placement.origin_core else {
            return Ok(None);
        };
        let global = [
            origin[0]
                .checked_add(tile.x as u16)
                .context("Safari x overflow")?,
            origin[1]
                .checked_add(tile.y as u16)
                .context("Safari y overflow")?,
        ];
        let mut found = self
            .habitats
            .iter()
            .filter(|habitat| habitat.contains(global));
        let habitat = found.next().context("Safari position has no habitat")?;
        ensure!(
            found.next().is_none(),
            "overlapping Safari position habitats"
        );
        Ok(Some(habitat))
    }
}

impl GameDataSet {
    /// Resolve the actual native Town Map landmark stored in caught data.
    pub(crate) fn safari_caught_location(&self, map_name: &str) -> Result<u8> {
        let id = self.pokegear_landmark_for_map(map_name)?.id;
        let location = u8::try_from(id)
            .with_context(|| format!("Safari landmark for {map_name} exceeds one byte"))?;
        ensure!(
            location <= 127,
            "Safari landmark for {map_name} exceeds caught-data's seven-bit field"
        );
        Ok(location)
    }

    /// Cross-check the typed expedition against this exact compiled pack.
    ///
    /// # Errors
    /// Rejects missing/changed configuration or wildlife references, unsafe
    /// returns and an unpermitted save inside an enabled preserve. Old packs
    /// with no config/session retain their prior fixture and save behavior.
    pub fn validate_saved_safari_references(&self, state: &GameState) -> Result<()> {
        self.validate_safari_prompt_references(state)?;
        safari::validate_saved_session(state).map_err(anyhow::Error::msg)?;
        let saved = safari::session(state).map_err(anyhow::Error::msg)?;
        let Some(config) = &self.safari_zone else {
            ensure!(
                saved.is_none(),
                "active Safari save requires its pack configuration"
            );
            return Ok(());
        };
        let OverworldMemory::Active { map_name, tile, .. } = &state.overworld else {
            ensure!(saved.is_none(), "Safari save requires an active overworld");
            return Ok(());
        };
        let Some(saved) = saved else {
            ensure!(
                !config.maps.contains_key(map_name) || config.gate_court.contains(map_name, *tile),
                "inactive expedition cannot enter the preserve"
            );
            if config.gate_court.contains(map_name, *tile) {
                let mut arrival = config.entry.clone();
                arrival.tile = *tile;
                validate_arrival(self, &arrival)?;
            }
            return Ok(());
        };
        ensure!(
            saved.zone_id == config.id && saved.return_to == config.return_to,
            "saved Safari configuration/return changed"
        );
        validate_arrival(self, &saved.return_to)?;
        if let Some(presentation) =
            crystal_core::systems::safari_catch_presentation::presentation(state)?
        {
            ensure!(
                presentation.nickname_required == self.nuzlocke_rules().require_capture_nickname,
                "saved Safari nickname policy disagrees with its pack"
            );
            if presentation.new_pokedex_entry {
                ensure!(
                    state
                        .flags
                        .is_engine_flag_set("ENGINE_POKEDEX")
                        .map_err(anyhow::Error::new)?,
                    "saved Safari Dex screen requires native Pokedex ownership"
                );
                let SafariPhase::Presenting { result, .. } = &saved.phase else {
                    anyhow::bail!("saved capture presentation has no result");
                };
                let safari::SafariResolution::Caught(stored) = &result.resolution else {
                    anyhow::bail!("saved capture presentation has no catch");
                };
                let entry = self
                    .pokedex_entries
                    .get(&stored.pokemon.species.id)
                    .context("Safari catch has no native Dex entry")?;
                ensure!(
                    presentation.dex_page < entry.pages.len(),
                    "saved Safari Dex page exceeds compiled entry"
                );
            }
        }
        if matches!(saved.phase, SafariPhase::Ending(_))
            && *map_name == config.return_to.map_name
            && *tile == config.return_to.tile
        {
            return Ok(());
        }
        ensure!(
            config.maps.contains_key(map_name),
            "active expedition is outside the preserve"
        );
        if let SafariPhase::Encounter(encounter) | SafariPhase::Presenting { encounter, .. } =
            &saved.phase
        {
            validate_encounter(self, config, encounter, state)?;
        }
        Ok(())
    }
}

fn validate_encounter(
    data: &GameDataSet,
    config: &SafariZone,
    encounter: &SafariEncounter,
    state: &GameState,
) -> Result<()> {
    data.validate_saved_pokemon_reference("safari.wildlife", &encounter.pokemon)?;
    let location = data.safari_caught_location(&encounter.origin_map_name)?;
    ensure!(
        encounter.caught_location == location,
        "saved Safari caught location disagrees with its native map"
    );
    if matches!(
        encounter.source,
        crystal_core::systems::safari::SafariEncounterSource::Fishing { .. }
    ) {
        return data.validate_safari_fishing_encounter(config, encounter, state);
    }
    let habitat = config
        .habitat_at(data, &encounter.origin_map_name, encounter.origin_tile)?
        .context("no Safari encounters in hospitality interiors")?;
    ensure!(
        encounter.habitat_id == habitat.id,
        "saved Safari habitat disagrees with actual geography"
    );
    let surface = match collision(data, &encounter.origin_map_name, encounter.origin_tile)? {
        "TALL_GRASS" => SafariSurface::Grass,
        "WATER" => SafariSurface::Water,
        _ => anyhow::bail!("saved Safari origin is not encounter terrain"),
    };
    ensure!(
        habitat
            .table(surface)
            .iter()
            .any(|slot| slot.species == encounter.pokemon.species.id
                && slot.level == encounter.pokemon.level
                && slot.flee_threshold == encounter.base_flee_threshold),
        "saved Safari species/level/flee rule is absent from its habitat"
    );
    Ok(())
}

pub(crate) fn collision<'a>(
    data: &'a GameDataSet,
    name: &str,
    tile: TilePosition,
) -> Result<&'a str> {
    let map = data.maps.get(name).context("Safari map missing")?;
    let (width, height) = native_bounds(map)?;
    ensure!(
        tile.x >= 0 && tile.y >= 0 && tile.x < width as i16 && tile.y < height as i16,
        "Safari tile outside map"
    );
    let (x, y) = (tile.x as usize, tile.y as usize);
    let block = map
        .blocks
        .get(y / 2 * map.attributes.width as usize + x / 2)
        .context("Safari metatile missing")?;
    data.tilesets
        .get(&map.attributes.tileset_name)
        .context("Safari tileset missing")?
        .collision
        .get(&format!("{block:02x}"))
        .and_then(|quadrants| quadrants.get(y % 2 * 2 + x % 2))
        .map(String::as_str)
        .context("Safari collision quadrant missing")
}

pub(crate) fn validate_arrival(data: &GameDataSet, arrival: &SafariReturn) -> Result<()> {
    ensure!(
        collision(data, &arrival.map_name, arrival.tile)? == "FLOOR",
        "Safari arrival must be safe FLOOR"
    );
    let map = &data.maps[&arrival.map_name];
    ensure!(
        !map.objects
            .iter()
            .any(|actor| actor.x as i16 == arrival.tile.x && actor.y as i16 == arrival.tile.y),
        "Safari arrival occupied by actor"
    );
    ensure!(
        !map.events
            .warps
            .iter()
            .any(|warp| warp.x as i16 == arrival.tile.x && warp.y as i16 == arrival.tile.y),
        "Safari arrival must be off warp triggers"
    );
    Ok(())
}

fn native_bounds(map: &MapModule) -> Result<(u16, u16)> {
    let width = map
        .attributes
        .width
        .checked_mul(2)
        .context("Safari map width overflow")?;
    let height = map
        .attributes
        .height
        .checked_mul(2)
        .context("Safari map height overflow")?;
    ensure!(
        width > 0 && height > 0 && width <= i16::MAX as u16 && height <= i16::MAX as u16,
        "Safari map is outside native tile coordinate bounds"
    );
    Ok((width, height))
}

fn token(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'),
        "malformed Safari identifier"
    );
    Ok(())
}

#[cfg(test)]
#[path = "safari_zone_tests.rs"]
mod tests;
