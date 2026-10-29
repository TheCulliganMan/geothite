//! Capture expeditions use the shared native mutation/replay path.

use anyhow::{Context, Result};

/// Read-only text for the same saved post-catch boundary as native graphics.
pub fn safari_catch_presentation_lines(
    source: &crate::RuntimeShellSnapshot,
) -> Option<Vec<String>> {
    use crystal_core::systems::safari_catch_presentation::SafariCatchStage as Stage;
    let saved = source.safari_catch_presentation.as_ref()?;
    let active = source.safari.as_ref()?;
    let crystal_core::systems::safari::SafariPhase::Presenting { result, .. } = &active.phase
    else {
        return None;
    };
    let crystal_core::systems::safari::SafariResolution::Caught(stored) = &result.resolution else {
        return None;
    };
    Some(match saved.stage {
        Stage::CaughtText => vec![
            format!("Gotcha! {}", stored.pokemon.nickname),
            "was caught!".into(),
        ],
        Stage::DexNotice => vec![
            format!("{}'s data", stored.pokemon.nickname),
            "was added to the".into(),
            "POKéDEX!".into(),
        ],
        Stage::DexEntry => vec!["POKéDEX".into(), stored.pokemon.nickname.clone()],
        Stage::NicknamePrompt => vec![
            "Give a nickname".into(),
            format!("to {}?", stored.pokemon.nickname),
        ],
        Stage::Naming => vec!["NICKNAME".into(), saved.name_draft.value.clone()],
        Stage::TransferText => match stored.location {
            crystal_core::models::CaptureStorageLocation::Party { .. } => {
                vec!["Joined your party.".into()]
            }
            crystal_core::models::CaptureStorageLocation::Pc { box_index, .. } => {
                let name = &source
                    .storage
                    .boxes
                    .iter()
                    .find(|pc| pc.index == box_index)?
                    .name;
                vec!["Sent to your PC.".into(), format!("Stored in {name}.")]
            }
        },
    })
}
use crystal_assets::{
    RuntimeDividerTrace, RuntimeMutationCommand, RuntimeMutationOutcome, RuntimeMutationResult,
    RuntimeSafariCommand, RuntimeSafariOutcome,
};
use crystal_core::multiplayer::game_state_checksum;
use crystal_core::random::RecordingDivider;
use crystal_core::systems::safari::SafariAction;

use crate::{CrystalRuntime, RecordedRuntimeMutation, RuntimeGameShell, RuntimeOverworldSession};

/// Native capture result lines shared by graphical, text and MCP presentation.
pub fn safari_result_lines(
    encounter: &crystal_core::systems::safari::SafariEncounter,
    result: &crystal_core::systems::safari::SafariResult,
) -> Vec<String> {
    use crystal_core::models::CaptureStorageLocation;
    use crystal_core::systems::safari::SafariResolution;
    if let SafariResolution::Caught(stored) = &result.resolution {
        return vec![
            encounter.pokemon.nickname.clone(),
            "was caught!".into(),
            match stored.location {
                CaptureStorageLocation::Party { .. } => "Joined your party.".into(),
                CaptureStorageLocation::Pc { box_index, .. } => {
                    format!("Sent to BOX{}.", box_index + 1)
                }
            },
        ];
    }
    let mut lines = vec![
        match result.action {
            SafariAction::Ball => "It broke free!",
            SafariAction::Bait => "Threw some BAIT.",
            SafariAction::Rock => "Threw a ROCK.",
            SafariAction::Run => "Got away safely!",
        }
        .to_string(),
    ];
    match result.resolution {
        SafariResolution::Fled => lines.push("The wild fled!".into()),
        SafariResolution::Continue => lines.push(
            match encounter.disposition {
                crystal_core::systems::safari::SafariDisposition::Eating { .. } => "It is eating.",
                crystal_core::systems::safari::SafariDisposition::Angry { .. } => "It is angry!",
                _ => "Watching you.",
            }
            .into(),
        ),
        _ => {}
    }
    lines
}

impl RuntimeOverworldSession {
    fn stage_safari_action(
        &mut self,
        runtime: &CrystalRuntime,
        action: SafariAction,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let outcome = runtime
            .data
            .resolve_safari_action(&mut state, action, &mut recording)?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        let state_checksum =
            game_state_checksum(&state).context("checksum staged native Safari action")?;
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::Safari(RuntimeSafariCommand::Act {
                action,
                divider_trace,
            }),
            state,
            overworld,
            outcome: RuntimeMutationOutcome {
                result: RuntimeMutationResult::SafariUpdated(RuntimeSafariOutcome::Action(outcome)),
                state_checksum,
            },
            divider_after: Some(divider_after),
        })
    }
}

impl RuntimeGameShell {
    /// Apply one saved post-catch screen or native naming input update.
    ///
    /// # Errors
    /// Invalid stages, drafts, storage or native journal replay reject atomically.
    pub fn update_safari_catch_presentation(
        &mut self,
        command: crystal_core::systems::safari_catch_presentation::SafariCatchCommand,
    ) -> Result<RuntimeSafariOutcome> {
        self.apply_safari_command(RuntimeSafariCommand::CatchPresentation(command))
    }
    /// Open the actual nearby ranger's native saved confirmation.
    ///
    /// # Errors
    /// Requires a configured ranger and no other native presentation owner.
    pub fn open_safari_prompt(&mut self) -> Result<RuntimeSafariOutcome> {
        self.apply_safari_command(RuntimeSafariCommand::OpenPrompt)
    }

    /// Navigate the saved ranger YES/NO cursor.
    ///
    /// # Errors
    /// Requires an active native ranger prompt.
    pub fn navigate_safari_prompt(
        &mut self,
        direction: crystal_core::world::map::Direction,
    ) -> Result<RuntimeSafariOutcome> {
        self.apply_safari_command(RuntimeSafariCommand::NavigatePrompt { direction })
    }

    /// Confirm the authoritative saved ranger selection once.
    ///
    /// # Errors
    /// Returns native context or transaction failures without charging on error.
    pub fn confirm_safari_prompt(&mut self) -> Result<RuntimeSafariOutcome> {
        self.apply_safari_command(RuntimeSafariCommand::ConfirmPrompt)
    }

    /// Cancel the native offer without spending or retiring a session.
    ///
    /// # Errors
    /// Returns native validation or journal errors.
    pub fn cancel_safari_prompt(&mut self) -> Result<RuntimeSafariOutcome> {
        self.apply_safari_command(RuntimeSafariCommand::CancelPrompt)
    }
    /// Move the authoritative capture cursor through the native mutation path.
    ///
    /// # Errors
    /// Requires an active capture command menu.
    pub fn navigate_safari(
        &mut self,
        direction: crystal_core::world::map::Direction,
    ) -> Result<RuntimeSafariOutcome> {
        self.apply_safari_command(RuntimeSafariCommand::Navigate { direction })
    }

    /// Confirm the saved command cursor using the native RNG staging path.
    ///
    /// # Errors
    /// Returns the same phase/resource/trace errors as a native Safari action.
    pub fn confirm_safari(&mut self) -> Result<RuntimeSafariOutcome> {
        let action = crystal_core::systems::safari::selected_action(&self.session.state)
            .map_err(anyhow::Error::new)?;
        self.use_safari_action(action)
    }
    /// Confirm/refuse the configured ranger through a real native transaction.
    ///
    /// # Errors
    /// Returns native config/context/warp errors. Entry refusals are typed
    /// outcomes; they do not debit money or create an expedition.
    pub fn admit_safari(&mut self, confirmed: bool) -> Result<RuntimeSafariOutcome> {
        self.apply_safari_command(RuntimeSafariCommand::Admit { confirmed })
    }

    /// Press a capture-only command using the live native hardware RNG source.
    ///
    /// # Errors
    /// Refuses invalid context/phase or full storage; staging/divider errors
    /// preserve resources. Recorded commands contain the exact consumed trace.
    pub fn use_safari_action(&mut self, action: SafariAction) -> Result<RuntimeSafariOutcome> {
        let recorded = self.session.stage_safari_action(&self.runtime, action)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        safari_outcome(mutation)
    }

    /// Release a fully presented native result exactly once.
    ///
    /// # Errors
    /// Requires the saved result phase and valid current pack references.
    pub fn acknowledge_safari_result(&mut self) -> Result<RuntimeSafariOutcome> {
        self.apply_safari_command(RuntimeSafariCommand::Acknowledge)
    }

    /// Queue retirement, retaining a pending capture result before return.
    ///
    /// # Errors
    /// Requires an active, structurally valid expedition.
    pub fn retire_safari(&mut self) -> Result<RuntimeSafariOutcome> {
        self.apply_safari_command(RuntimeSafariCommand::Retire)
    }

    /// Commit the safe native return after presentation releases input.
    ///
    /// # Errors
    /// Requires ending and a valid unchanged return; native failures roll back.
    pub fn return_from_safari(&mut self) -> Result<RuntimeSafariOutcome> {
        self.apply_safari_command(RuntimeSafariCommand::Return)
    }

    fn apply_safari_command(
        &mut self,
        command: RuntimeSafariCommand,
    ) -> Result<RuntimeSafariOutcome> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::Safari(command))?;
        safari_outcome(mutation)
    }
}

fn safari_outcome(mutation: RuntimeMutationOutcome) -> Result<RuntimeSafariOutcome> {
    let RuntimeMutationResult::SafariUpdated(outcome) = mutation.result else {
        anyhow::bail!("native Safari mutation returned another result kind");
    };
    Ok(outcome)
}
