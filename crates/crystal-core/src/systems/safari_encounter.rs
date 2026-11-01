//! Atomic capture-only actions over the actual session, party and current box.

use crate::battle::capture::{self, CaptureAttemptContext, CaptureRules, CaptureWobbleProbability};
use crate::models::pokemon::CaughtData;
use crate::random::BattleRandomSource;
use crate::state::GameState;
use crate::systems::safari::{
    self, SafariAction, SafariDisposition, SafariEncounter, SafariEndReason, SafariError,
    SafariPhase, SafariResolution, SafariResult,
};

/// Resolve one Safari command, retaining its result until presentation finishes.
/// Uses session balls, real native catch/wobble RNG, party/current-PC storage
/// and Pokédex; no player/enemy attack, bag consumption, PP, EXP or money turn.
///
/// # Errors
/// Refuses inactive/wrong-phase commands, malformed saved state or capture rules.
/// Full storage rejects Ball before consuming any resources or randomness.
/// Successful mutations are staged; a saved pending result cannot resolve twice.
pub fn resolve_action(
    state: &mut GameState,
    action: SafariAction,
    rules: &CaptureRules,
    wobble_probabilities: &[CaptureWobbleProbability],
    rng: &mut dyn BattleRandomSource,
) -> Result<SafariResult, SafariError> {
    safari::validate_saved_session(state)?;
    let mut session = safari::session(state)?.ok_or(SafariError::Inactive)?;
    let SafariPhase::Encounter(mut encounter) = session.phase else {
        return Err(SafariError::WrongPhase("no available encounter command"));
    };
    // Validate every fallible Ball precondition before debit or the first draw.
    let final_catch_rate = if action == SafariAction::Ball {
        state.storage.validate_metadata().map_err(safari::invalid)?;
        if !state
            .storage
            .has_capture_space_in_box(state.current_pc_box)
            .map_err(safari::invalid)?
        {
            return Err(SafariError::StorageFull);
        }
        let rate = safari_catch_rate(&encounter, rules)?;
        capture::wobble_chance_for_rate(rate, wobble_probabilities).map_err(capture_error)?;
        Some(rate)
    } else {
        None
    };
    encounter.resolved_actions = encounter
        .resolved_actions
        .checked_add(1)
        .ok_or_else(|| safari::invalid("Safari action counter overflow"))?;
    let mut staged = state.clone();
    let mut outcome = None;
    let mut resolution = SafariResolution::Continue;
    match action {
        SafariAction::Ball => {
            session.balls_remaining -= 1;
            let rate =
                final_catch_rate.ok_or_else(|| safari::invalid("missing validated catch rate"))?;
            // The explicit Safari context has no held item or player combatant.
            // Carry starts clear for this original capture-only mode.
            let mut capture = capture::resolve_capture_rate_with_battle_rng(
                rate,
                false,
                wobble_probabilities,
                rng,
            )
            .map_err(capture_error)?;
            capture.ball_id = Some("SAFARI_BALL".to_string());
            if capture.caught {
                let mut pokemon = encounter.pokemon.clone();
                pokemon.original_trainer_name = if staged.player_name.is_empty() {
                    "PLAYER".to_string()
                } else {
                    staged.player_name.clone()
                };
                pokemon.original_trainer_id = staged.player_id;
                pokemon.caught_data = Some(CaughtData {
                    level: pokemon.level & 0x3f,
                    time_of_day: Some(staged.time.time_of_day),
                    original_trainer_gender: staged.player_gender,
                    location: encounter.caught_location,
                });
                let stored = capture::complete_captured_pokemon(
                    &capture,
                    &mut staged.storage,
                    staged.current_pc_box,
                    &mut staged.pokedex,
                    pokemon,
                )
                .map_err(safari::invalid)?
                .ok_or_else(|| safari::invalid("successful Safari catch was not stored"))?;
                staged.sync_party_from_storage();
                resolution = SafariResolution::Caught(Box::new(stored));
            }
            outcome = Some(capture);
        }
        SafariAction::Bait => {
            encounter.disposition = SafariDisposition::Eating {
                responses_remaining: 2 + (rng.crystal_random_byte(false) & 3),
            }
        }
        SafariAction::Rock => {
            encounter.disposition = SafariDisposition::Angry {
                responses_remaining: 2 + (rng.crystal_random_byte(false) & 3),
            }
        }
        SafariAction::Run => resolution = SafariResolution::Ran,
    }
    if resolution == SafariResolution::Continue {
        let threshold = encounter.disposition.scale(encounter.base_flee_threshold);
        if rng.crystal_random_byte(false) < threshold {
            resolution = SafariResolution::Fled;
        }
        encounter.disposition = encounter.disposition.after_response();
    }
    let result = SafariResult {
        action,
        capture: outcome,
        resolution,
    };
    session.command_cursor = match action {
        SafariAction::Ball => 0,
        SafariAction::Bait => 1,
        SafariAction::Rock => 2,
        SafariAction::Run => 3,
    };
    session.phase = SafariPhase::Presenting {
        encounter,
        result: result.clone(),
    };
    if session.balls_remaining == 0 {
        session.pending_end = Some(SafariEndReason::BallsExhausted);
    }
    safari::set_session(&mut staged, &session)?;
    *state = staged;
    Ok(result)
}

fn safari_catch_rate(encounter: &SafariEncounter, rules: &CaptureRules) -> Result<u8, SafariError> {
    let rule = rules
        .ball_rules
        .get("SAFARI_BALL")
        .ok_or_else(|| safari::invalid("missing native Safari Ball rule"))?;
    if rule.multiplier_numerator != 3
        || rule.multiplier_denominator != 2
        || (!rule.battle_type.is_empty() && rule.battle_type != "BATTLETYPE_SAFARI")
        || rule.skip_hp_calc
        || rule.use_heavy_ball_weight_modifier
        || rule.use_level_ball_multiplier
        || rule.require_same_species
        || rule.require_same_gender
        || rule.require_fast_species
        || rules.guaranteed_capture_balls.contains("SAFARI_BALL")
    {
        return Err(safari::invalid(
            "mn_safari_v1 requires the native unconditional 3/2 Safari Ball",
        ));
    }
    let mut context = CaptureAttemptContext::wild("SAFARI_BALL");
    context.battle_type = "BATTLETYPE_SAFARI".to_string();
    // The validated unconditional rule does not inspect a player. Reusing the
    // wildlife reference for that unused parameter avoids a fictitious party
    // combatant while retaining the native multiplier and full-HP calculation.
    let base = encounter
        .disposition
        .scale(encounter.pokemon.species.catch_rate);
    let mut native_divisor = encounter.pokemon.max_hp.wrapping_mul(3);
    if native_divisor & 0xff00 != 0 {
        native_divisor >>= 2;
    }
    if native_divisor & 0x00ff == 0 {
        return Err(safari::invalid(
            "wildlife HP produces a zero native catch divisor",
        ));
    }
    let ball_rate = capture::apply_ball_multiplier_with_base_rate(
        base,
        "SAFARI_BALL",
        &encounter.pokemon,
        &encounter.pokemon,
        &context,
        rules,
    )
    .map_err(capture_error)?;
    // Preserve native integer rounding after applying the authored disposition
    // to the species base rate, before its Safari Ball and full-HP calculation.
    Ok(capture::compute_hp_adjusted_catch_rate(
        ball_rate.rate,
        encounter.pokemon.hp,
        encounter.pokemon.max_hp,
    )
    .max(1))
}

fn capture_error(error: capture::CaptureError) -> SafariError {
    SafariError::Capture(error.to_string())
}
