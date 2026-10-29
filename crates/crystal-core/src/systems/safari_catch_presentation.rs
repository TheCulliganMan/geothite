//! Saved capture completion, retaining the real catch until all screens close.

use serde::{Deserialize, Serialize};

use crate::models::CaptureStorageLocation;
use crate::state::GameState;
use crate::systems::safari::{self, SafariError, SafariPhase, SafariResolution};

const KEY: &str = "GEOTHITE_SAFARI_CATCH_PRESENTATION_V1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafariCatchStage {
    CaughtText,
    DexNotice,
    DexEntry,
    NicknamePrompt,
    Naming,
    TransferText,
}

/// The production naming screen's saved draft and native nine-by-five cursor.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafariNameDraft {
    pub value: String,
    pub cursor_column: usize,
    pub cursor_row: usize,
    pub uppercase: bool,
}

impl Default for SafariNameDraft {
    fn default() -> Self {
        Self {
            value: String::new(),
            cursor_column: 0,
            cursor_row: 0,
            uppercase: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafariCatchPresentation {
    pub version: u8,
    pub stage: SafariCatchStage,
    pub new_pokedex_entry: bool,
    pub nickname_required: bool,
    pub selected_yes: bool,
    pub dex_page: usize,
    pub nickname_page: usize,
    pub name_draft: SafariNameDraft,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum SafariCatchCommand {
    Advance,
    SelectNickname { selected_yes: bool },
    ConfirmNickname,
    UpdateNameDraft { draft: SafariNameDraft },
    FinishName,
    SetDexPage { page: usize },
    SetNicknamePage { page: usize },
}

/// Read the saved screen owner without changing capture or storage.
///
/// # Errors
/// Rejects malformed/versioned data and invalid native cursor/name bounds.
pub fn presentation(state: &GameState) -> Result<Option<SafariCatchPresentation>, SafariError> {
    let Some(encoded) = state.script_runtime.memory.get(KEY) else {
        return Ok(None);
    };
    let saved: SafariCatchPresentation = serde_json::from_str(encoded)
        .map_err(|error| safari::invalid(format!("capture presentation encoding: {error}")))?;
    validate_shape(&saved)?;
    Ok(Some(saved))
}

fn validate_shape(saved: &SafariCatchPresentation) -> Result<(), SafariError> {
    if saved.version != 1
        || saved.dex_page >= 16
        || saved.nickname_page >= 16
        || saved.name_draft.cursor_column >= super::name_entry::NAME_ENTRY_COLUMNS
        || saved.name_draft.cursor_row >= super::name_entry::NAME_ENTRY_ROWS
        || saved.name_draft.value.chars().count() > 10
        || saved
            .name_draft
            .value
            .chars()
            .any(|ch| ch.is_control() || ch == '@')
        || (matches!(
            saved.stage,
            SafariCatchStage::DexEntry | SafariCatchStage::DexNotice
        ) && !saved.new_pokedex_entry)
    {
        return Err(safari::invalid("invalid native capture presentation state"));
    }
    Ok(())
}

/// Validate this optional owner during ordinary binary Continue/checksum checks.
///
/// # Errors
/// A screen requires the same still-pending, actually stored Safari catch.
pub fn validate_saved_presentation(state: &GameState) -> Result<(), SafariError> {
    if presentation(state)?.is_some() {
        let active = safari::session(state)?.ok_or(SafariError::Inactive)?;
        if !matches!(active.phase, SafariPhase::Presenting { result, .. }
            if matches!(result.resolution, SafariResolution::Caught(_)))
        {
            return Err(safari::invalid(
                "capture screen has no pending stored Safari catch",
            ));
        }
    }
    Ok(())
}

fn write(state: &mut GameState, saved: &SafariCatchPresentation) -> Result<(), SafariError> {
    validate_shape(saved)?;
    let encoded =
        serde_json::to_string(saved).map_err(|error| safari::invalid(error.to_string()))?;
    state.script_runtime.memory.insert(KEY.into(), encoded);
    Ok(())
}

/// Begin completion only after the actual catch has reached party/PC storage.
/// The adapter supplies native Pokedex ownership/novelty and nickname policy.
///
/// # Errors
/// Rejects another presentation, invalid capture/storage, or missing catch.
pub fn begin(
    state: &mut GameState,
    new_pokedex_entry: bool,
    nickname_required: bool,
) -> Result<(), SafariError> {
    safari::validate_saved_session(state)?;
    if presentation(state)?.is_some() {
        return Err(safari::invalid("capture presentation already active"));
    }
    let saved = SafariCatchPresentation {
        version: 1,
        stage: SafariCatchStage::CaughtText,
        new_pokedex_entry,
        nickname_required,
        selected_yes: true,
        dex_page: 0,
        nickname_page: 0,
        name_draft: Default::default(),
    };
    let mut staged = state.clone();
    write(&mut staged, &saved)?;
    validate_saved_presentation(&staged)?;
    *state = staged;
    Ok(())
}

/// Advance one saved capture screen or apply one native naming-screen change.
/// Never rerolls a Ball, charges, changes party membership or warps the player.
///
/// # Errors
/// Wrong stages, invalid drafts and missing storage reject atomically. Renaming
/// updates the real party/PC slot and retained result together, including box
/// metadata. Final acknowledgement alone releases pending budget exhaustion.
pub fn apply(state: &mut GameState, command: SafariCatchCommand) -> Result<(), SafariError> {
    use SafariCatchStage as Stage;
    safari::validate_saved_session(state)?;
    let mut saved =
        presentation(state)?.ok_or_else(|| safari::invalid("no capture presentation"))?;
    let mut staged = state.clone();
    match command {
        SafariCatchCommand::Advance => {
            saved.stage = match saved.stage {
                Stage::CaughtText if saved.new_pokedex_entry => Stage::DexNotice,
                Stage::CaughtText | Stage::DexEntry => Stage::NicknamePrompt,
                Stage::DexNotice => Stage::DexEntry,
                Stage::TransferText => {
                    staged.script_runtime.memory.remove(KEY);
                    safari::acknowledge_result(&mut staged)?;
                    *state = staged;
                    return Ok(());
                }
                _ => {
                    return Err(SafariError::WrongPhase(
                        "capture screen requires its native choice or name",
                    ));
                }
            };
        }
        SafariCatchCommand::SelectNickname { selected_yes }
            if saved.stage == Stage::NicknamePrompt =>
        {
            saved.selected_yes = selected_yes;
        }
        SafariCatchCommand::ConfirmNickname if saved.stage == Stage::NicknamePrompt => {
            saved.stage = if saved.selected_yes || saved.nickname_required {
                Stage::Naming
            } else {
                Stage::TransferText
            };
        }
        SafariCatchCommand::UpdateNameDraft { draft } if saved.stage == Stage::Naming => {
            saved.name_draft = draft;
        }
        SafariCatchCommand::FinishName if saved.stage == Stage::Naming => {
            if saved.name_draft.value.trim() != saved.name_draft.value {
                return Err(safari::invalid("nickname must be exact native text"));
            }
            if !saved.name_draft.value.is_empty() {
                rename_caught(&mut staged, &saved.name_draft.value)?;
            }
            saved.stage = Stage::TransferText;
        }
        SafariCatchCommand::SetDexPage { page } if saved.stage == Stage::DexEntry => {
            saved.dex_page = page;
        }
        SafariCatchCommand::SetNicknamePage { page } if saved.stage == Stage::NicknamePrompt => {
            saved.nickname_page = page;
        }
        _ => {
            return Err(SafariError::WrongPhase(
                "command does not belong to this capture screen",
            ));
        }
    }
    write(&mut staged, &saved)?;
    safari::validate_saved_session(&staged)?;
    *state = staged;
    Ok(())
}

fn rename_caught(state: &mut GameState, nickname: &str) -> Result<(), SafariError> {
    let mut active = safari::session(state)?.ok_or(SafariError::Inactive)?;
    let SafariPhase::Presenting { result, .. } = &mut active.phase else {
        return Err(SafariError::WrongPhase("no caught result"));
    };
    let SafariResolution::Caught(stored) = &mut result.resolution else {
        return Err(SafariError::WrongPhase("no stored catch"));
    };
    stored.pokemon.nickname = nickname.into();
    match stored.location {
        CaptureStorageLocation::Party { slot } => {
            let target = state
                .storage
                .party
                .pokemon
                .get_mut(slot)
                .ok_or_else(|| safari::invalid("party catch slot missing"))?;
            *target = Some(stored.pokemon.clone());
            state.sync_party_from_storage();
        }
        CaptureStorageLocation::Pc { box_index, slot } => {
            let pc = state
                .storage
                .pc_boxes
                .get_mut(box_index)
                .ok_or_else(|| safari::invalid("PC catch box missing"))?;
            if slot >= pc.pokemon.len() {
                return Err(safari::invalid("PC catch slot missing"));
            }
            pc.set_slot(slot, Some(stored.pokemon.clone()));
        }
    }
    safari::set_session(state, &active)
}
