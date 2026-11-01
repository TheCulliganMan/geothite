//! Actual capture wildlife from native preserve grass and Surf geography.

use anyhow::{Context, Result};
use crystal_core::battle::start::{
    wild_dvs_from_crystal_random, wild_held_item_from_crystal_random,
};
use crystal_core::random::{CrystalRandom, DividerSource};
use crystal_core::state::GameState;
use crystal_core::systems::map_context::SpawnMemoryUpdate;
use crystal_core::systems::safari::{self, SafariDisposition, SafariEncounter, SafariPhase};
use crystal_core::world::movement::MovementMode;
use crystal_core::world::session::OverworldSession;

use crate::safari_zone::collision;
use crate::{GameDataSet, SafariSurface};

impl GameDataSet {
    pub(crate) fn start_safari_encounter_after_step<S>(
        &self,
        state: &mut GameState,
        overworld: &OverworldSession,
        rng: &mut CrystalRandom<&mut S>,
    ) -> Result<Option<SafariEncounter>>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        let Some(active) = safari::session(state).map_err(anyhow::Error::new)? else {
            return Ok(None);
        };
        if active.phase != SafariPhase::Exploring {
            return Ok(None);
        }
        let zone = self
            .safari_zone
            .as_ref()
            .context("active Safari has no habitat configuration")?;
        if state.wild_encounter_cooldown > 0 {
            state.wild_encounter_cooldown -= 1;
            if state.wild_encounter_cooldown > 0 {
                return Ok(None);
            }
        }
        if state
            .flags
            .is_engine_flag_set("STATUSFLAGS_NO_WILD_ENCOUNTERS_F")
            .map_err(anyhow::Error::new)?
        {
            return Ok(None);
        }
        let Some(habitat) = zone.habitat_at(self, &overworld.map.name, overworld.player.tile)?
        else {
            return Ok(None);
        };
        let (surface, rate) = match collision(self, &overworld.map.name, overworld.player.tile)? {
            "TALL_GRASS" => (SafariSurface::Grass, habitat.grass_rate),
            "WATER"
                if matches!(
                    overworld.player.mode,
                    MovementMode::Surf | MovementMode::SurfPika
                ) =>
            {
                (SafariSurface::Water, habitat.water_rate)
            }
            _ => return Ok(None),
        };
        if rng
            .random(false)
            .map_err(|error| anyhow::anyhow!("Safari encounter rate divider: {error}"))?
            .value
            >= rate
        {
            return Ok(None);
        }
        // Validated tables total100. Rejection samples a uniform percentile
        // rather than biasing weights with an unrestricted byte modulo100.
        let ticket = loop {
            let byte = rng
                .random(false)
                .map_err(|error| anyhow::anyhow!("Safari habitat divider: {error}"))?
                .value;
            if byte < 200 {
                break byte % 100;
            }
        };
        let mut cumulative = 0u16;
        let slot = habitat
            .table(surface)
            .iter()
            .find(|slot| {
                cumulative += u16::from(slot.weight);
                u16::from(ticket) < cumulative
            })
            .context("Safari percentile is absent from its habitat table")?;
        // Respect native repel crossing behavior, without the combat challenge
        // mod's multi-Pokémon party generation or opponent scaling.
        if state.repel_steps_remaining > 0
            && state
                .storage
                .party
                .pokemon
                .iter()
                .flatten()
                .find(|pokemon| !pokemon.is_egg && pokemon.hp > 0)
                .is_some_and(|lead| slot.level < lead.level)
        {
            return Ok(None);
        }
        let species = self
            .pokemon
            .get(&slot.species)
            .context("Safari wildlife species missing")?;
        let item = wild_held_item_from_crystal_random(species, "BATTLETYPE_SAFARI", rng)
            .map_err(|error| anyhow::anyhow!("Safari native held item: {error:?}"))?;
        let dvs = wild_dvs_from_crystal_random(rng)
            .map_err(|error| anyhow::anyhow!("Safari native DVs: {error:?}"))?;
        let mut pokemon = self.create_pokemon(&slot.species, slot.level, dvs)?;
        pokemon.original_trainer_name = "WILD".to_string();
        pokemon.original_trainer_id = 0;
        pokemon.item = item;
        let encounter = SafariEncounter {
            pokemon,
            habitat_id: habitat.id.clone(),
            origin_map_name: overworld.map.name.clone(),
            origin_tile: overworld.player.tile,
            caught_location: self.safari_caught_location(&overworld.map.name)?,
            base_flee_threshold: slot.flee_threshold,
            disposition: SafariDisposition::Neutral,
            resolved_actions: 0,
            source: Default::default(),
        };
        // Commit the actual native movement position with this same staged
        // frame; do not reconstruct or inject a separate encounter placement.
        self.commit_overworld_snapshot(state, overworld, SpawnMemoryUpdate::Preserve);
        safari::begin_encounter(state, encounter.clone()).map_err(anyhow::Error::new)?;
        self.validate_saved_safari_references(state)?;
        Ok(Some(encounter))
    }
}
