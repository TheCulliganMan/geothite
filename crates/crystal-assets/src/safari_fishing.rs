//! Native rod-table wildlife with an explicit, saved shore/water contact.

use anyhow::{Context, Result, ensure};
use crystal_core::battle::start::{
    wild_dvs_from_crystal_random, wild_held_item_from_crystal_random,
};
use crystal_core::random::{CrystalRandom, DividerSource};
use crystal_core::state::{GameState, OverworldMemory};
use crystal_core::systems::map_context::SpawnMemoryUpdate;
use crystal_core::systems::safari::{
    self, SafariDisposition, SafariEncounter, SafariEncounterSource, SafariPhase,
};
use crystal_core::world::fishing::{FishingSession, resolve_fishing_slot};
use crystal_core::world::map::TilePosition;
use crystal_core::world::movement::{MovementMode, StepOptions, checked_move_by_stride};
use crystal_core::world::session::OverworldSession;

use crate::safari_zone::collision;
use crate::{GameDataSet, SafariZone};

impl GameDataSet {
    pub(crate) fn start_safari_fishing_encounter<S>(
        &self,
        state: &mut GameState,
        overworld: &OverworldSession,
        cast: &FishingSession,
        slot_roll: u8,
        water_tile: TilePosition,
        rng: &mut CrystalRandom<&mut S>,
    ) -> Result<SafariEncounter>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        ensure!(
            safari::session(state)?.is_some_and(|active| active.phase == SafariPhase::Exploring),
            "Safari fishing requires an exploring permit"
        );
        let zone = self
            .safari_zone
            .as_ref()
            .context("Safari fishing configuration missing")?;
        let habitat = zone
            .habitat_at(self, &overworld.map.name, water_tile)?
            .context("Safari fishing requires exterior habitat water")?;
        let fish = cast
            .outcome
            .encounter
            .as_ref()
            .context("Safari fishing has no native bite")?;
        let species = self
            .pokemon
            .get(&fish.species)
            .context("native rod species missing")?;
        let item = wild_held_item_from_crystal_random(species, "BATTLETYPE_SAFARI", rng)
            .map_err(|error| anyhow::anyhow!("Safari rod held item: {error:?}"))?;
        let dvs = wild_dvs_from_crystal_random(rng)
            .map_err(|error| anyhow::anyhow!("Safari rod DVs: {error:?}"))?;
        let mut pokemon = self.create_pokemon(&fish.species, fish.level, dvs)?;
        pokemon.original_trainer_name = "WILD".into();
        pokemon.original_trainer_id = 0;
        pokemon.item = item;
        let wild = SafariEncounter {
            pokemon,
            habitat_id: habitat.id.clone(),
            origin_map_name: overworld.map.name.clone(),
            origin_tile: overworld.player.tile,
            caught_location: self.safari_caught_location(&overworld.map.name)?,
            base_flee_threshold: zone.fishing_flee_threshold,
            disposition: SafariDisposition::Neutral,
            resolved_actions: 0,
            source: SafariEncounterSource::Fishing {
                water_tile,
                rod: cast.rod.clone(),
                group: cast.group.clone().context("native fishing group missing")?,
                time_of_day: state.time.time_of_day,
                slot_roll,
            },
        };
        self.commit_overworld_snapshot(state, overworld, SpawnMemoryUpdate::Preserve);
        safari::begin_encounter(state, wild.clone())?;
        self.validate_saved_safari_references(state)?;
        Ok(wild)
    }

    pub(crate) fn validate_safari_fishing_encounter(
        &self,
        zone: &SafariZone,
        wild: &SafariEncounter,
        state: &GameState,
    ) -> Result<()> {
        let SafariEncounterSource::Fishing {
            water_tile,
            rod,
            group,
            time_of_day,
            slot_roll,
        } = &wild.source
        else {
            anyhow::bail!("Safari fishing source is missing");
        };
        let OverworldMemory::Active { facing, mode, .. } = &state.overworld else {
            anyhow::bail!("Safari fishing has no native shore position");
        };
        ensure!(
            !matches!(mode, MovementMode::Surf | MovementMode::SurfPika),
            "Safari fishing origin cannot be surfing"
        );
        ensure!(
            checked_move_by_stride(
                wild.origin_tile,
                *facing,
                StepOptions::default().stride_tiles
            ) == Some(*water_tile),
            "saved Safari fishing contact disagrees with actual facing"
        );
        ensure!(
            collision(self, &wild.origin_map_name, *water_tile)? == "WATER",
            "Safari fishing contact is not real water"
        );
        let habitat = zone
            .habitat_at(self, &wild.origin_map_name, *water_tile)?
            .context("Safari fishing contact has no exterior habitat")?;
        ensure!(
            wild.habitat_id == habitat.id
                && wild.base_flee_threshold == zone.fishing_flee_threshold,
            "Safari fishing habitat or flee rule changed"
        );
        let base_group = self
            .map_fishing_group(&wild.origin_map_name)?
            .context("Safari map has no native fishing group")?;
        ensure!(
            group == base_group
                || self
                    .fishing
                    .swarm_rules
                    .values()
                    .any(|rule| rule.base_group == base_group && rule.swarm_group == *group),
            "Safari fishing group does not belong to its actual map"
        );
        let expected = resolve_fishing_slot(&self.fishing, group, rod, *time_of_day, *slot_roll)
            .map_err(anyhow::Error::new)?;
        ensure!(
            expected.species == wild.pokemon.species.id && expected.level == wild.pokemon.level,
            "Safari wildlife disagrees with its native rod/time/slot"
        );
        Ok(())
    }
}
