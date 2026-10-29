//! Saved capture expeditions, independent of ordinary combat and Bug Contest.
//!
//! A single versioned, typed value uses the existing saved extension envelope.
//! Absent expeditions add no bytes to legacy binary saves or their checksums.
//! Runtime adapters must validate map/config references and commit admission
//! together with the real native warp before installing a new session.

use serde::{Deserialize, Serialize};

use crate::battle::capture::{CaptureOutcome, StoredCapture};
use crate::models::Pokemon;
use crate::models::box_storage::CaptureStorageLocation;
use crate::state::{BattleMemory, GameState};
use crate::world::map::{Direction, TilePosition};

const SESSION_KEY: &str = "GEOTHITE_SAFARI_SESSION_V1";
pub const BALL_BUDGET: u8 = 30;
pub const STEP_BUDGET: u16 = 500;
pub const ADMISSION_FEE: u32 = 500;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SafariError {
    #[error("invalid Safari state: {0}")]
    InvalidState(String),
    #[error("no active Safari expedition")]
    Inactive,
    #[error("Safari command is unavailable during {0}")]
    WrongPhase(&'static str),
    #[error("party and current PC box have no capture space")]
    StorageFull,
    #[error("Safari capture: {0}")]
    Capture(String),
}

/// A safe ordinary-walking return, resolved against the active pack by runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafariReturn {
    pub map_name: String,
    pub tile: TilePosition,
    pub facing: Direction,
}

/// Original Minnesota balance choices, not cartridge Safari behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum SafariDisposition {
    Neutral,
    Eating { responses_remaining: u8 },
    Angry { responses_remaining: u8 },
}

impl SafariDisposition {
    pub(crate) fn scale(self, value: u8) -> u8 {
        let value = u16::from(value);
        let scaled = match self {
            Self::Neutral => value,
            Self::Eating { .. } => value / 2,
            Self::Angry { .. } => value * 2,
        };
        scaled.clamp(1, 255) as u8
    }

    pub(crate) fn after_response(self) -> Self {
        match self {
            Self::Neutral => Self::Neutral,
            Self::Eating {
                responses_remaining: 1,
            }
            | Self::Angry {
                responses_remaining: 1,
            } => Self::Neutral,
            Self::Eating {
                responses_remaining,
            } => Self::Eating {
                responses_remaining: responses_remaining - 1,
            },
            Self::Angry {
                responses_remaining,
            } => Self::Angry {
                responses_remaining: responses_remaining - 1,
            },
        }
    }

    fn validate(self) -> Result<(), SafariError> {
        match self {
            Self::Neutral => Ok(()),
            Self::Eating {
                responses_remaining: 1..=5,
            }
            | Self::Angry {
                responses_remaining: 1..=5,
            } => Ok(()),
            _ => Err(invalid("disposition duration must be in 1..=5")),
        }
    }
}

/// Native encounter provenance, including a rod's separate shore/water contact.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum SafariEncounterSource {
    #[default]
    Field,
    Fishing {
        water_tile: TilePosition,
        rod: String,
        group: String,
        time_of_day: crate::world::encounters::TimeOfDay,
        slot_roll: u8,
    },
}

impl SafariEncounterSource {
    fn is_field(&self) -> bool {
        matches!(self, Self::Field)
    }
}

/// Saved wildlife includes its real shore contact when caught by fishing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafariEncounter {
    pub pokemon: Pokemon,
    pub habitat_id: String,
    pub origin_map_name: String,
    pub origin_tile: TilePosition,
    pub caught_location: u8,
    pub base_flee_threshold: u8,
    pub disposition: SafariDisposition,
    pub resolved_actions: u32,
    #[serde(default, skip_serializing_if = "SafariEncounterSource::is_field")]
    pub source: SafariEncounterSource,
}

impl SafariEncounter {
    pub(crate) fn validate(&self) -> Result<(), SafariError> {
        self.pokemon.validate_saved_state().map_err(invalid)?;
        self.pokemon
            .species
            .validate_compiled_shape()
            .map_err(invalid)?;
        validate_token(&self.habitat_id)?;
        validate_token(&self.origin_map_name)?;
        validate_tile(self.origin_tile)?;
        self.disposition.validate()?;
        if let SafariEncounterSource::Fishing {
            water_tile,
            rod,
            group,
            ..
        } = &self.source
        {
            validate_tile(*water_tile)?;
            validate_token(group)?;
            crate::world::fishing::validate_rod(rod).map_err(|error| invalid(error.to_string()))?;
            let distance = (i32::from(water_tile.x) - i32::from(self.origin_tile.x)).abs()
                + (i32::from(water_tile.y) - i32::from(self.origin_tile.y)).abs();
            if distance != 1 {
                return Err(invalid(
                    "fishing wildlife requires an adjacent real water contact",
                ));
            }
        }
        if self.pokemon.is_egg
            || self.pokemon.hp == 0
            || self.pokemon.hp != self.pokemon.max_hp
            || self.pokemon.status.is_some()
            || self.pokemon.sleep_turns != 0
        {
            return Err(invalid(
                "capture-only wildlife must have full HP and no status",
            ));
        }
        if self.base_flee_threshold == 0 || self.caught_location > 127 {
            return Err(invalid("flee threshold or caught location is out of range"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum SafariAction {
    Ball,
    Bait,
    Rock,
    Run,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum SafariResolution {
    Continue,
    Caught(Box<StoredCapture>),
    Fled,
    Ran,
}

/// A committed action awaiting presentation. Repeating input cannot reroll it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafariResult {
    pub action: SafariAction,
    pub capture: Option<CaptureOutcome>,
    pub resolution: SafariResolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum SafariEndReason {
    BallsExhausted,
    StepsExhausted,
    Retired,
    LeftPreserve,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum SafariPhase {
    Exploring,
    Encounter(SafariEncounter),
    Presenting {
        encounter: SafariEncounter,
        result: SafariResult,
    },
    Ending(SafariEndReason),
}

/// Full saved session. The RNG remains in the authoritative game random state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafariSession {
    pub version: u8,
    pub zone_id: String,
    pub balls_remaining: u8,
    pub steps_remaining: u16,
    pub return_to: SafariReturn,
    pub phase: SafariPhase,
    pub pending_end: Option<SafariEndReason>,
    /// Authoritative two-by-two capture menu cursor, retained across Continue.
    #[serde(default)]
    pub command_cursor: u8,
}

impl SafariSession {
    /// Prepare an unpaid session for a staged native admission transaction.
    ///
    /// # Errors
    /// Rejects malformed zone or return identifiers/coordinates. This does not
    /// verify Surf, storage, money, map collision or warp success, or admit anyone.
    pub fn new(zone_id: String, return_to: SafariReturn) -> Result<Self, SafariError> {
        let session = Self {
            version: 1,
            zone_id,
            balls_remaining: BALL_BUDGET,
            steps_remaining: STEP_BUDGET,
            return_to,
            phase: SafariPhase::Exploring,
            pending_end: None,
            command_cursor: 0,
        };
        session.validate()?;
        Ok(session)
    }

    pub(crate) fn validate(&self) -> Result<(), SafariError> {
        validate_token(&self.zone_id)?;
        validate_token(&self.return_to.map_name)?;
        validate_tile(self.return_to.tile)?;
        if self.command_cursor > 3 {
            return Err(invalid("Safari command cursor is outside its four choices"));
        }
        if self.version != 1
            || self.balls_remaining > BALL_BUDGET
            || self.steps_remaining > STEP_BUDGET
        {
            return Err(invalid("unsupported version or expedition budgets"));
        }
        match &self.phase {
            SafariPhase::Exploring | SafariPhase::Encounter(_)
                if self.balls_remaining == 0
                    || self.steps_remaining == 0
                    || self.pending_end.is_some() =>
            {
                return Err(invalid(
                    "live exploration/encounter cannot have exhausted or ending budgets",
                ));
            }
            SafariPhase::Ending(reason) => {
                if self.pending_end.is_some() {
                    return Err(invalid("ending session must not have another pending end"));
                }
                validate_end(*reason, self)?;
            }
            _ => {}
        }
        match &self.phase {
            SafariPhase::Encounter(encounter) => encounter.validate()?,
            SafariPhase::Presenting { encounter, result } => {
                encounter.validate()?;
                validate_result(encounter, result)?;
                if self.steps_remaining == 0 {
                    return Err(invalid("encounter result cannot follow exhausted movement"));
                }
                if self.balls_remaining == 0
                    && self.pending_end != Some(SafariEndReason::BallsExhausted)
                {
                    return Err(invalid("last-ball result must queue exhaustion"));
                }
                if self.balls_remaining == 0 && result.action != SafariAction::Ball {
                    return Err(invalid("only a Ball action can exhaust the ball budget"));
                }
                if let Some(reason) = self.pending_end {
                    validate_end(reason, self)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Read the complete typed session. A missing key means inactive; malformed or
/// unknown versions fail instead of resetting paid resources.
///
/// # Errors
/// Returns invalid state for damaged encoding, impossible budgets or phases.
pub fn session(state: &GameState) -> Result<Option<SafariSession>, SafariError> {
    let Some(encoded) = state.script_runtime.memory.get(SESSION_KEY) else {
        return Ok(None);
    };
    let session: SafariSession = serde_json::from_str(encoded)
        .map_err(|error| invalid(format!("session encoding: {error}")))?;
    session.validate()?;
    Ok(Some(session))
}

/// Persist an already staged session without changing the binary save schema.
///
/// # Errors
/// Rejects invalid session state and incompatible native battle ownership.
pub fn set_session(state: &mut GameState, session: &SafariSession) -> Result<(), SafariError> {
    session.validate()?;
    validate_ownership(state)?;
    validate_stored_result(state, session)?;
    validate_origin(state, session)?;
    let encoded = serde_json::to_string(session).map_err(|error| invalid(error.to_string()))?;
    state
        .script_runtime
        .memory
        .insert(SESSION_KEY.to_string(), encoded);
    Ok(())
}

/// Validate the extension during ordinary save/checksum loading.
///
/// # Errors
/// Rejects malformed sessions, simultaneous combat/contest/link ownership, or
/// a pending caught result missing from its real party/current-PC storage slot.
pub fn validate_saved_session(state: &GameState) -> Result<(), SafariError> {
    if let Some(session) = session(state)? {
        validate_ownership(state)?;
        validate_stored_result(state, &session)?;
        validate_origin(state, &session)?;
    }
    super::safari_catch_presentation::validate_saved_presentation(state)?;
    Ok(())
}

/// Debit one successful native travel commit, never an input or animation tick.
/// Runtime must invoke this only at its authoritative travel mutation boundary.
///
/// # Errors
/// Refuses movement during an encounter, result or ending; invalid saves fail.
pub fn record_successful_travel(
    state: &mut GameState,
) -> Result<Option<SafariEndReason>, SafariError> {
    let Some(mut active) = session(state)? else {
        return Ok(None);
    };
    if active.phase != SafariPhase::Exploring {
        return Err(SafariError::WrongPhase("capture or return"));
    }
    active.steps_remaining -= 1;
    let end = (active.steps_remaining == 0).then_some(SafariEndReason::StepsExhausted);
    if let Some(reason) = end {
        active.phase = SafariPhase::Ending(reason);
    }
    set_session(state, &active)?;
    Ok(end)
}

/// Commit a materialized habitat encounter while retaining expedition budgets.
///
/// # Errors
/// Requires exploring and valid full-HP wildlife. Runtime must also prove the
/// habitat/species/level and origin belong to the active pack's Safari config.
pub fn begin_encounter(
    state: &mut GameState,
    encounter: SafariEncounter,
) -> Result<(), SafariError> {
    let mut active = session(state)?.ok_or(SafariError::Inactive)?;
    if active.phase != SafariPhase::Exploring {
        return Err(SafariError::WrongPhase("capture or return"));
    }
    encounter.validate()?;
    if encounter.disposition != SafariDisposition::Neutral || encounter.resolved_actions != 0 {
        return Err(invalid(
            "new encounter must start neutral with no resolved actions",
        ));
    }
    active.phase = SafariPhase::Encounter(encounter);
    active.command_cursor = 0;
    set_session(state, &active)
}

/// Move the native capture-menu cursor without spending a turn or randomness.
///
/// # Errors
/// Requires the command phase and valid saved state. Directions clamp at the
/// menu's two-by-two boundaries; result/ending screens cannot move the cursor.
pub fn navigate_commands(state: &mut GameState, direction: Direction) -> Result<u8, SafariError> {
    validate_saved_session(state)?;
    let mut active = session(state)?.ok_or(SafariError::Inactive)?;
    if !matches!(active.phase, SafariPhase::Encounter(_)) {
        return Err(SafariError::WrongPhase("capture commands unavailable"));
    }
    let cursor = active.command_cursor;
    active.command_cursor = match direction {
        Direction::Up if cursor >= 2 => cursor - 2,
        Direction::Down if cursor < 2 => cursor + 2,
        Direction::Left if cursor % 2 == 1 => cursor - 1,
        Direction::Right if cursor % 2 == 0 => cursor + 1,
        _ => cursor,
    };
    set_session(state, &active)?;
    Ok(active.command_cursor)
}

/// Read the selected native command; renderer-local cursors are never used.
///
/// # Errors
/// Requires a valid command-phase session.
pub fn selected_action(state: &GameState) -> Result<SafariAction, SafariError> {
    validate_saved_session(state)?;
    let active = session(state)?.ok_or(SafariError::Inactive)?;
    if !matches!(active.phase, SafariPhase::Encounter(_)) {
        return Err(SafariError::WrongPhase("capture commands unavailable"));
    }
    Ok(match active.command_cursor {
        0 => SafariAction::Ball,
        1 => SafariAction::Bait,
        2 => SafariAction::Rock,
        3 => SafariAction::Run,
        _ => return Err(invalid("invalid Safari command cursor")),
    })
}

/// Release a fully presented result once. Last-ball catches remain stored before
/// this schedules return; it never throws again or performs a return warp.
///
/// # Errors
/// Requires a saved pending result and rejects damaged session/storage state.
pub fn acknowledge_result(state: &mut GameState) -> Result<(), SafariError> {
    validate_saved_session(state)?;
    if super::safari_catch_presentation::presentation(state)?.is_some() {
        return Err(SafariError::WrongPhase(
            "finish native capture screens before releasing the catch",
        ));
    }
    let mut active = session(state)?.ok_or(SafariError::Inactive)?;
    let SafariPhase::Presenting { encounter, result } = active.phase else {
        return Err(SafariError::WrongPhase("no result to acknowledge"));
    };
    active.phase = if let Some(reason) = active.pending_end.take() {
        SafariPhase::Ending(reason)
    } else if result.resolution == SafariResolution::Continue {
        SafariPhase::Encounter(encounter)
    } else {
        SafariPhase::Exploring
    };
    set_session(state, &active)
}

/// Request a safe return, deferring it while a committed result owns input.
///
/// # Errors
/// Invalid state fails; exhaustion reasons require matching zero budgets.
pub fn request_end(state: &mut GameState, reason: SafariEndReason) -> Result<(), SafariError> {
    let mut active = session(state)?.ok_or(SafariError::Inactive)?;
    if active.pending_end.is_some() || matches!(active.phase, SafariPhase::Ending(_)) {
        return Ok(());
    }
    validate_end(reason, &active)?;
    match active.phase {
        SafariPhase::Presenting { .. } => {
            if active.pending_end.is_none() {
                active.pending_end = Some(reason);
            }
        }
        SafariPhase::Ending(_) => return Ok(()),
        _ => {
            active.phase = SafariPhase::Ending(reason);
        }
    }
    set_session(state, &active)
}

/// Clear the permit only after the native runtime has committed its safe return.
///
/// # Errors
/// Requires ending and an exact matching ordinary-walking arrival/facing.
pub fn complete_return(state: &mut GameState) -> Result<(), SafariError> {
    validate_saved_session(state)?;
    let active = session(state)?.ok_or(SafariError::Inactive)?;
    if !matches!(active.phase, SafariPhase::Ending(_)) {
        return Err(SafariError::WrongPhase("return not ready"));
    }
    match &state.overworld {
        crate::state::OverworldMemory::Active {
            map_name,
            tile,
            facing,
            mode: crate::world::movement::MovementMode::Normal,
        } if *map_name == active.return_to.map_name
            && *tile == active.return_to.tile
            && *facing == active.return_to.facing => {}
        _ => return Err(invalid("return must land at the validated walking arrival")),
    }
    state.script_runtime.memory.remove(SESSION_KEY);
    Ok(())
}

/// Settle an expedition after its pack adapter commits an outward transition.
///
/// The adapter must verify that the source belongs to the preserve and the
/// destination lies outside it. Native travel keeps its actual destination;
/// this only clears expedition resources, preserving catches and other state.
///
/// # Errors
/// Requires exploration or a settled ending, and an exact committed arrival.
/// A capture or unresolved result can never be discarded by travel.
pub fn complete_departure(
    state: &mut GameState,
    destination: &str,
    tile: TilePosition,
    facing: Direction,
) -> Result<(), SafariError> {
    validate_saved_session(state)?;
    validate_token(destination)?;
    validate_tile(tile)?;
    let active = session(state)?.ok_or(SafariError::Inactive)?;
    if !matches!(
        active.phase,
        SafariPhase::Exploring | SafariPhase::Ending(_)
    ) {
        return Err(SafariError::WrongPhase(
            "departure cannot discard capture input",
        ));
    }
    if !matches!(&state.overworld,
        crate::state::OverworldMemory::Active {map_name,tile:actual,facing:direction,..}
        if map_name == destination && *actual == tile && *direction == facing)
    {
        return Err(invalid("departure requires its committed native arrival"));
    }
    state.script_runtime.memory.remove(SESSION_KEY);
    Ok(())
}

fn validate_ownership(state: &GameState) -> Result<(), SafariError> {
    if state.battle != BattleMemory::Inactive
        || state.script_runtime.active_battle_combat.is_some()
        || state.bug_contest.timer_active
        || !state.bug_contest.party_backup.is_empty()
        || state.link_session.link_mode != 0
    {
        return Err(invalid(
            "Safari cannot share combat, contest or link ownership",
        ));
    }
    Ok(())
}

fn validate_stored_result(state: &GameState, session: &SafariSession) -> Result<(), SafariError> {
    if let SafariPhase::Presenting {
        encounter,
        result:
            SafariResult {
                resolution: SafariResolution::Caught(stored),
                ..
            },
    } = &session.phase
    {
        let pokemon = match stored.location {
            CaptureStorageLocation::Party { slot } => state
                .storage
                .party
                .pokemon
                .get(slot)
                .and_then(Option::as_ref),
            CaptureStorageLocation::Pc { box_index, slot } => state
                .storage
                .pc_boxes
                .get(box_index)
                .and_then(|pc| pc.pokemon.get(slot))
                .and_then(Option::as_ref),
        };
        if pokemon != Some(&stored.pokemon) {
            return Err(invalid(
                "pending Safari catch must match its committed storage slot",
            ));
        }
        let caught = stored
            .pokemon
            .caught_data
            .as_ref()
            .ok_or_else(|| invalid("pending catch requires native caught metadata"))?;
        let mut expected = encounter.pokemon.clone();
        expected.nickname = stored.pokemon.nickname.clone();
        expected.original_trainer_name = if state.player_name.is_empty() {
            "PLAYER".to_string()
        } else {
            state.player_name.clone()
        };
        expected.original_trainer_id = state.player_id;
        expected.caught_data = Some(crate::models::pokemon::CaughtData {
            level: expected.level & 0x3f,
            time_of_day: caught.time_of_day,
            original_trainer_gender: state.player_gender,
            location: encounter.caught_location,
        });
        if caught.time_of_day.is_none()
            || stored.pokemon != expected
            || !state.pokedex.has_caught(&expected.species.id)
        {
            return Err(invalid(
                "pending catch must preserve wildlife, trainer, location and Pokédex data",
            ));
        }
    }
    Ok(())
}

fn validate_origin(state: &GameState, session: &SafariSession) -> Result<(), SafariError> {
    if matches!(state.overworld, crate::state::OverworldMemory::Inactive) {
        return Err(invalid(
            "an active Safari session requires an overworld position",
        ));
    }
    if let SafariPhase::Encounter(encounter) | SafariPhase::Presenting { encounter, .. } =
        &session.phase
    {
        match &state.overworld {
            crate::state::OverworldMemory::Active { map_name, tile, .. }
                if *map_name == encounter.origin_map_name && *tile == encounter.origin_tile => {}
            _ => {
                return Err(invalid(
                    "wildlife origin must match the authoritative player position",
                ));
            }
        }
    }
    Ok(())
}

fn validate_result(encounter: &SafariEncounter, result: &SafariResult) -> Result<(), SafariError> {
    if encounter.resolved_actions == 0 {
        return Err(invalid("pending result requires a committed action"));
    }
    match (&result.action, &result.capture, &result.resolution) {
        (SafariAction::Ball, Some(capture), resolution) => {
            if capture.blocked
                || capture.storage_full
                || capture.ball_id.as_deref() != Some("SAFARI_BALL")
                || capture.final_catch_rate == 0
                || capture.wobble_count > 3
                || capture.animation_shakes > 4
                || (capture.caught && (capture.wobble_count != 3 || capture.animation_shakes != 4))
                || (!capture.caught && capture.animation_shakes != capture.wobble_count)
            {
                return Err(invalid("invalid Safari capture outcome"));
            }
            match resolution {
                SafariResolution::Caught(stored)
                    if capture.caught
                        && stored.pokemon.species == encounter.pokemon.species
                        && stored.pokemon.level == encounter.pokemon.level
                        && stored.pokemon.dvs == encounter.pokemon.dvs =>
                {
                    stored.pokemon.validate_saved_state().map_err(invalid)?
                }
                SafariResolution::Continue | SafariResolution::Fled if !capture.caught => {}
                _ => return Err(invalid("capture result disagrees with stored resolution")),
            }
        }
        (
            SafariAction::Bait | SafariAction::Rock,
            None,
            SafariResolution::Continue | SafariResolution::Fled,
        )
        | (SafariAction::Run, None, SafariResolution::Ran) => {}
        _ => return Err(invalid("action and result do not agree")),
    }
    Ok(())
}

fn validate_end(reason: SafariEndReason, session: &SafariSession) -> Result<(), SafariError> {
    if (reason == SafariEndReason::BallsExhausted && session.balls_remaining != 0)
        || (reason == SafariEndReason::StepsExhausted && session.steps_remaining != 0)
    {
        return Err(invalid("exhaustion reason requires an exhausted budget"));
    }
    if session.balls_remaining == 0 && reason != SafariEndReason::BallsExhausted {
        return Err(invalid("empty balls require ball exhaustion"));
    }
    if session.steps_remaining == 0 && reason != SafariEndReason::StepsExhausted {
        return Err(invalid("empty steps require step exhaustion"));
    }
    Ok(())
}

fn validate_token(value: &str) -> Result<(), SafariError> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(invalid("empty or malformed Safari identifier"));
    }
    Ok(())
}

fn validate_tile(tile: TilePosition) -> Result<(), SafariError> {
    if tile.x < 0 || tile.y < 0 {
        return Err(invalid("negative Safari tile"));
    }
    Ok(())
}

pub(crate) fn invalid(message: impl Into<String>) -> SafariError {
    SafariError::InvalidState(message.into())
}

#[cfg(test)]
#[path = "safari_tests.rs"]
mod tests;
