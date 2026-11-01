//! Native saved ranger offers, confirmations and retirement.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::state::{GameState, OverworldMemory};
use crystal_core::systems::safari::{self, SafariEndReason, SafariPhase};
use crystal_core::systems::safari_prompt::{self, SafariPrompt, SafariPromptKind};
use crystal_core::world::map::TilePosition;
use crystal_core::world::session::OverworldSession;

use crate::{GameDataSet, RuntimeSafariOutcome};

impl GameDataSet {
    /// Resolve the actual facing ranger's role, without opening a local menu.
    ///
    /// # Errors
    /// Rejects damaged expedition state or invalid native object coordinates.
    pub fn safari_ranger_at_player(
        &self,
        state: &GameState,
        map: &OverworldSession,
    ) -> Result<Option<SafariPromptKind>> {
        let Some(zone) = &self.safari_zone else {
            return Ok(None);
        };
        let (dx, dy) = map.player.facing.delta();
        let target = TilePosition::new(
            map.player
                .tile
                .x
                .checked_add(dx)
                .context("ranger x overflow")?,
            map.player
                .tile
                .y
                .checked_add(dy)
                .context("ranger y overflow")?,
        );
        let Some((_, actor)) = map.visible_object_at_checked(target)? else {
            return Ok(None);
        };
        let active = safari::session(state).map_err(anyhow::Error::new)?;
        if active.is_none()
            && map.map.name == zone.admission_map
            && actor.script == zone.admission_script
        {
            return Ok(Some(SafariPromptKind::Admission));
        }
        if active.is_some_and(|active| active.phase == SafariPhase::Exploring)
            && zone.maps.contains_key(&map.map.name)
            && zone.retire_scripts.contains(&actor.script)
        {
            return Ok(Some(SafariPromptKind::Retire));
        }
        Ok(None)
    }

    pub(crate) fn open_safari_prompt(
        &self,
        state: &mut GameState,
        map: &OverworldSession,
    ) -> Result<()> {
        self.validate_saved_safari_references(state)?;
        ensure!(
            safari_prompt::prompt(state)
                .map_err(anyhow::Error::new)?
                .is_none(),
            "ranger confirmation already owns input"
        );
        let script = &state.script_runtime;
        ensure!(
            !script.player_input_locked
                && !script.all_input_locked
                && !script.window_open
                && !script.text_window_open
                && script.active_menu.is_none()
                && script.pending_yes_no.is_none()
                && script.next_script.is_none()
                && script.command_queue.is_empty(),
            "another native presentation owns ranger input"
        );
        let kind = self
            .safari_ranger_at_player(state, map)?
            .context("no configured facing Safari ranger")?;
        let zone = self.safari_zone.as_ref().context("Safari config missing")?;
        let prompt = SafariPrompt {
            version: 1,
            zone_id: zone.id.clone(),
            map_name: map.map.name.clone(),
            tile: map.player.tile,
            facing: map.player.facing,
            kind,
            selected_yes: true,
        };
        safari_prompt::set_prompt(state, &prompt).map_err(anyhow::Error::new)?;
        self.validate_safari_prompt_references(state)
    }

    pub(crate) fn confirm_safari_prompt(
        &self,
        state: &mut GameState,
        map: &mut OverworldSession,
        music: &BTreeSet<String>,
    ) -> Result<RuntimeSafariOutcome> {
        self.validate_safari_prompt_references(state)?;
        let prompt = safari_prompt::prompt(state)
            .map_err(anyhow::Error::new)?
            .context("no ranger confirmation")?;
        safari_prompt::clear_prompt(state);
        if !prompt.selected_yes {
            return Ok(RuntimeSafariOutcome::PromptClosed);
        }
        match prompt.kind {
            SafariPromptKind::Admission => Ok(RuntimeSafariOutcome::Admission(
                self.admit_safari(state, map, true, music)?,
            )),
            SafariPromptKind::Retire => {
                safari::request_end(state, SafariEndReason::Retired).map_err(anyhow::Error::new)?;
                self.return_from_safari(state, map, music)?;
                Ok(RuntimeSafariOutcome::Returned)
            }
        }
    }

    pub(crate) fn validate_safari_prompt_references(&self, state: &GameState) -> Result<()> {
        let Some(prompt) = safari_prompt::prompt(state).map_err(anyhow::Error::new)? else {
            return Ok(());
        };
        let zone = self
            .safari_zone
            .as_ref()
            .context("saved ranger offer requires its zone config")?;
        ensure!(prompt.zone_id == zone.id, "saved ranger offer zone changed");
        ensure!(
            matches!(&state.overworld,OverworldMemory::Active{map_name,tile,facing,..} if *map_name==prompt.map_name && *tile==prompt.tile && *facing==prompt.facing),
            "saved ranger offer origin changed"
        );
        let map = self
            .maps
            .get(&prompt.map_name)
            .context("saved ranger map missing")?;
        let (dx, dy) = prompt.facing.delta();
        let x = prompt
            .tile
            .x
            .checked_add(dx)
            .context("ranger target x overflow")?;
        let y = prompt
            .tile
            .y
            .checked_add(dy)
            .context("ranger target y overflow")?;
        ensure!(
            map.objects.iter().any(|actor| actor.x as i16 == x
                && actor.y as i16 == y
                && match prompt.kind {
                    SafariPromptKind::Admission =>
                        prompt.map_name == zone.admission_map
                            && actor.script == zone.admission_script,
                    SafariPromptKind::Retire =>
                        zone.maps.contains_key(&prompt.map_name)
                            && zone.retire_scripts.contains(&actor.script),
                }),
            "saved ranger actor/role is absent"
        );
        Ok(())
    }
}
