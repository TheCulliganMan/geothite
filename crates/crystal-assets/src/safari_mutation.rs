//! Safari actions share the native mutation/checksum/divider replay boundary.

use std::collections::BTreeSet;

use anyhow::{Context, Result};
use crystal_core::random::{DividerSource, ExactBattleRandom, ReplayDivider};
use crystal_core::state::GameState;
use crystal_core::systems::safari::{self, SafariAction, SafariEndReason, SafariResult};
use crystal_core::systems::safari_encounter;
use crystal_core::world::session::OverworldSession;
use serde::{Deserialize, Serialize};

use crate::{GameDataSet, RuntimeDividerTrace, SafariAdmissionOutcome};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeSafariCommand {
    CatchPresentation(crystal_core::systems::safari_catch_presentation::SafariCatchCommand),
    OpenPrompt,
    NavigatePrompt {
        direction: crystal_core::world::map::Direction,
    },
    ConfirmPrompt,
    CancelPrompt,
    Navigate {
        direction: crystal_core::world::map::Direction,
    },
    Admit {
        confirmed: bool,
    },
    Act {
        action: SafariAction,
        divider_trace: RuntimeDividerTrace,
    },
    Acknowledge,
    Retire,
    Return,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeSafariOutcome {
    CapturePresentationUpdated,
    PromptOpened,
    PromptChanged,
    PromptClosed,
    CursorMoved(u8),
    Admission(SafariAdmissionOutcome),
    Action(SafariResult),
    Acknowledged,
    Retired,
    Returned,
}

impl GameDataSet {
    pub(crate) fn require_safari_input_ownership(
        &self,
        state: &GameState,
        command: &crate::RuntimeMutationCommand,
    ) -> Result<()> {
        use crate::RuntimeMutationCommand as Command;
        let prompt = crystal_core::systems::safari_prompt::prompt(state)
            .map_err(anyhow::Error::new)?
            .is_some();
        let capture = safari::session(state)
            .map_err(anyhow::Error::new)?
            .is_some_and(|active| {
                matches!(
                    active.phase,
                    safari::SafariPhase::Encounter(_) | safari::SafariPhase::Presenting { .. }
                )
            });
        if !prompt && !capture {
            return Ok(());
        }
        anyhow::ensure!(
            matches!(
                command,
                Command::Safari(_)
                    | Command::ApplyOverworldInput(_)
                    | Command::AdvanceGameTimerVBlanks(_)
                    | Command::SetGameTimerCounting(_)
                    | Command::SetGameLogicPaused(_)
                    | Command::UpdateClockFromDatetime(_)
                    | Command::SetManualClockTime(_)
                    | Command::UpdateTime
                    // Consuming already committed presentation notifications
                    // does not resume scripts or authorize another gameplay act.
                    | Command::DrainScriptEventQueue(_)
            ),
            "native Safari capture/ranger input owns gameplay commands"
        );
        Ok(())
    }
    /// Resolve a native capture-only action using recorded hardware divider
    /// samples, committing the real random state with the result and storage.
    ///
    /// # Errors
    /// Rejects incompatible map/session state, invalid commands or unavailable
    /// resources. Divider exhaustion and validation failures roll back state.
    pub fn resolve_safari_action<S>(
        &self,
        state: &mut GameState,
        action: SafariAction,
        divider: &mut S,
    ) -> Result<SafariResult>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        self.validate_saved_safari_references(state)?;
        self.safari_zone
            .as_ref()
            .context("Safari command requires its native zone")?;
        let prior_species = match safari::session(state)?
            .context("Safari session missing")?
            .phase
        {
            safari::SafariPhase::Encounter(wild) => Some(wild.pokemon.species.id),
            _ => None,
        };
        let newly_caught = prior_species
            .as_deref()
            .is_some_and(|species| !state.pokedex.has_caught(species));
        let mut staged = state.clone();
        let mut rng = ExactBattleRandom::new(staged.random_state, divider);
        let outcome = safari_encounter::resolve_action(
            &mut staged,
            action,
            &self.capture_rules,
            &self.capture_wobble_probabilities,
            &mut rng,
        )
        .map_err(anyhow::Error::new)?;
        if let Some(error) = rng.divider_error() {
            anyhow::bail!("Safari native divider: {error}");
        }
        staged.random_state = rng.state();
        if matches!(outcome.resolution, safari::SafariResolution::Caught(_)) {
            let owns_dex = staged
                .flags
                .is_engine_flag_set("ENGINE_POKEDEX")
                .map_err(anyhow::Error::new)?;
            crystal_core::systems::safari_catch_presentation::begin(
                &mut staged,
                newly_caught && owns_dex,
                self.nuzlocke_rules().require_capture_nickname,
            )?;
        }
        self.validate_saved_safari_references(&staged)?;
        *state = staged;
        Ok(outcome)
    }

    pub(crate) fn apply_safari_command(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        command: RuntimeSafariCommand,
        music_ids: &BTreeSet<String>,
    ) -> Result<RuntimeSafariOutcome> {
        let mut staged = state.clone();
        let mut staged_map = overworld.clone();
        let outcome = match command {
            RuntimeSafariCommand::CatchPresentation(command) => {
                crystal_core::systems::safari_catch_presentation::apply(&mut staged, command)?;
                if safari::session(&staged)?
                    .is_some_and(|active| active.phase == safari::SafariPhase::Exploring)
                {
                    staged.wild_encounter_cooldown = 5;
                }
                RuntimeSafariOutcome::CapturePresentationUpdated
            }
            RuntimeSafariCommand::OpenPrompt => {
                self.open_safari_prompt(&mut staged, &staged_map)?;
                RuntimeSafariOutcome::PromptOpened
            }
            RuntimeSafariCommand::NavigatePrompt { direction } => {
                crystal_core::systems::safari_prompt::navigate(&mut staged, direction)
                    .map_err(anyhow::Error::new)?;
                RuntimeSafariOutcome::PromptChanged
            }
            RuntimeSafariCommand::ConfirmPrompt => {
                self.confirm_safari_prompt(&mut staged, &mut staged_map, music_ids)?
            }
            RuntimeSafariCommand::CancelPrompt => {
                crystal_core::systems::safari_prompt::clear_prompt(&mut staged);
                RuntimeSafariOutcome::PromptClosed
            }
            RuntimeSafariCommand::Navigate { direction } => RuntimeSafariOutcome::CursorMoved(
                safari::navigate_commands(&mut staged, direction).map_err(anyhow::Error::new)?,
            ),
            RuntimeSafariCommand::Admit { confirmed } => RuntimeSafariOutcome::Admission(
                self.admit_safari(&mut staged, &mut staged_map, confirmed, music_ids)?,
            ),
            RuntimeSafariCommand::Act {
                action,
                divider_trace,
            } => {
                let mut divider = ReplayDivider::new(divider_trace.samples.iter().copied());
                let result = self.resolve_safari_action(&mut staged, action, &mut divider)?;
                crate::require_consumed_divider_trace("Safari action", &divider)?;
                RuntimeSafariOutcome::Action(result)
            }
            RuntimeSafariCommand::Acknowledge => {
                if crystal_core::systems::safari_catch_presentation::presentation(&staged)?
                    .is_some()
                {
                    crystal_core::systems::safari_catch_presentation::apply(
                        &mut staged, crystal_core::systems::safari_catch_presentation::SafariCatchCommand::Advance,
                    )?;
                } else {
                    safari::acknowledge_result(&mut staged).map_err(anyhow::Error::new)?;
                }
                if safari::session(&staged)
                    .map_err(anyhow::Error::new)?
                    .is_some_and(|active| active.phase == safari::SafariPhase::Exploring)
                {
                    staged.wild_encounter_cooldown = 5;
                }
                RuntimeSafariOutcome::Acknowledged
            }
            RuntimeSafariCommand::Retire => {
                safari::request_end(&mut staged, SafariEndReason::Retired)
                    .map_err(anyhow::Error::new)?;
                RuntimeSafariOutcome::Retired
            }
            RuntimeSafariCommand::Return => {
                self.return_from_safari(&mut staged, &mut staged_map, music_ids)?;
                RuntimeSafariOutcome::Returned
            }
        };
        self.validate_saved_safari_references(&staged)?;
        *state = staged;
        *overworld = staged_map;
        Ok(outcome)
    }
}
