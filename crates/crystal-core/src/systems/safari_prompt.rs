//! Saved native ranger confirmation, independent of renderer-local cursors.

use serde::{Deserialize, Serialize};

use crate::state::GameState;
use crate::systems::safari::{self, SafariError, SafariPhase};
use crate::world::map::{Direction, TilePosition};

const KEY: &str = "GEOTHITE_SAFARI_PROMPT_V1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum SafariPromptKind {
    Admission,
    Retire,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafariPrompt {
    pub version: u8,
    pub zone_id: String,
    pub map_name: String,
    pub tile: TilePosition,
    pub facing: Direction,
    pub kind: SafariPromptKind,
    pub selected_yes: bool,
}

/// Read a saved confirmation without resetting malformed input ownership.
///
/// # Errors
/// Rejects unknown versions, malformed identifiers and incompatible sessions.
pub fn prompt(state: &GameState) -> Result<Option<SafariPrompt>, SafariError> {
    let Some(encoded) = state.script_runtime.memory.get(KEY) else {
        return Ok(None);
    };
    let prompt: SafariPrompt = serde_json::from_str(encoded)
        .map_err(|error| safari::invalid(format!("ranger prompt: {error}")))?;
    validate(state, &prompt)?;
    Ok(Some(prompt))
}

/// Save the complete authoritative ranger offer and selection.
///
/// # Errors
/// Rejects invalid context or a prompt that conflicts with capture ownership.
pub fn set_prompt(state: &mut GameState, prompt: &SafariPrompt) -> Result<(), SafariError> {
    validate(state, prompt)?;
    state.script_runtime.memory.insert(
        KEY.to_string(),
        serde_json::to_string(prompt).map_err(|error| safari::invalid(error.to_string()))?,
    );
    Ok(())
}

/// Release the native confirmation owner. Resources are settled by its staged
/// admission/retirement transaction, not by clearing this value.
pub fn clear_prompt(state: &mut GameState) {
    state.script_runtime.memory.remove(KEY);
}

/// Navigate the native YES/NO selection without spending resources or RNG.
///
/// # Errors
/// Requires a valid saved ranger prompt.
pub fn navigate(state: &mut GameState, direction: Direction) -> Result<(), SafariError> {
    let mut current = prompt(state)?.ok_or(SafariError::WrongPhase("no ranger confirmation"))?;
    match direction {
        Direction::Up => current.selected_yes = true,
        Direction::Down => current.selected_yes = false,
        _ => {}
    }
    set_prompt(state, &current)
}

fn validate(state: &GameState, prompt: &SafariPrompt) -> Result<(), SafariError> {
    if prompt.version != 1
        || prompt.tile.x < 0
        || prompt.tile.y < 0
        || [&prompt.zone_id, &prompt.map_name].iter().any(|value| {
            value.is_empty()
                || !value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        })
    {
        return Err(safari::invalid("invalid saved ranger confirmation"));
    }
    let session = safari::session(state)?;
    match (prompt.kind, session) {
        (SafariPromptKind::Admission, None) => Ok(()),
        (SafariPromptKind::Retire, Some(active))
            if active.zone_id == prompt.zone_id && active.phase == SafariPhase::Exploring =>
        {
            Ok(())
        }
        _ => Err(safari::invalid(
            "ranger prompt conflicts with expedition ownership",
        )),
    }
}
