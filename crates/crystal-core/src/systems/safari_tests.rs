#[path = "safari_catch_presentation_tests.rs"]
mod capture_presentation;

use crate::multiplayer;
use crate::state::GameState;
use std::collections::VecDeque;

use super::{
    self as safari, SafariAction, SafariDisposition, SafariEncounter, SafariEndReason, SafariError,
    SafariPhase, SafariResolution, SafariReturn, SafariSession,
};
use crate::battle::capture::{CaptureBallRule, CaptureRules, CaptureWobbleProbability};
use crate::models::{
    BaseStats, CaptureStorageLocation, Dv, MAX_BOX_MONS, PARTY_SIZE, Pokemon, PokemonSpecies,
};
use crate::random::BattleRandomSource;
use crate::save::{self, SaveModpackIdentity};
use crate::state::OverworldMemory;
use crate::systems::safari_encounter;
use crate::world::map::{Direction, TilePosition};
use crate::world::movement::MovementMode;

#[derive(Default)]
struct Draws {
    bytes: VecDeque<u8>,
    carries: Vec<bool>,
}

impl Draws {
    fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.iter().copied().collect(),
            carries: Vec::new(),
        }
    }
}

impl BattleRandomSource for Draws {
    fn battle_random_byte(&mut self) -> u8 {
        panic!("Safari must use the authoritative Crystal RNG operation");
    }
    fn crystal_random_byte(&mut self, carry: bool) -> u8 {
        self.carries.push(carry);
        self.bytes
            .pop_front()
            .expect("only declared native random draws")
    }
}

fn wildlife() -> Pokemon {
    let mut species =
        PokemonSpecies::new_for_tests("GILLPUP", BaseStats::new(55, 55, 55, 55, 55, 55));
    species.catch_rate = 120;
    species.int_id = 1;
    Pokemon::new_for_tests(species, 24, Dv::from_non_hp(10, 10, 10, 10))
}

fn encounter() -> SafariEncounter {
    SafariEncounter {
        pokemon: wildlife(),
        habitat_id: "reed_marsh".into(),
        origin_map_name: "MnBoundarySafariC00R00".into(),
        origin_tile: TilePosition::new(2, 2),
        caught_location: 42,
        base_flee_threshold: 64,
        disposition: SafariDisposition::Neutral,
        resolved_actions: 0,
        source: Default::default(),
    }
}

fn exploring() -> GameState {
    let mut state = GameState::reset_wram_for_new_game();
    state.overworld = OverworldMemory::Active {
        map_name: "MnBoundarySafariC00R00".into(),
        tile: TilePosition::new(2, 2),
        facing: Direction::Down,
        mode: MovementMode::Normal,
    };
    let session = SafariSession::new(
        "boundary_waters".into(),
        SafariReturn {
            map_name: "MnBoundaryWaters".into(),
            tile: TilePosition::new(12, 12),
            facing: Direction::Down,
        },
    )
    .expect("prepared session");
    safari::set_session(&mut state, &session).expect("test setup, not native admission");
    state
}

fn capturing() -> GameState {
    let mut state = exploring();
    safari::begin_encounter(&mut state, encounter()).expect("materialized fixture wildlife");
    state
}

fn rules() -> CaptureRules {
    CaptureRules {
        ball_rules: [(
            "SAFARI_BALL".into(),
            CaptureBallRule {
                multiplier_numerator: 3,
                multiplier_denominator: 2,
                battle_type: String::new(),
                skip_hp_calc: false,
                use_heavy_ball_weight_modifier: false,
                use_level_ball_multiplier: false,
                require_same_species: false,
                require_same_gender: false,
                require_fast_species: false,
            },
        )]
        .into(),
        ..CaptureRules::default()
    }
}

fn action(
    state: &mut GameState,
    action: SafariAction,
    rng: &mut Draws,
) -> Result<super::SafariResult, SafariError> {
    safari_encounter::resolve_action(
        state,
        action,
        &rules(),
        &[CaptureWobbleProbability {
            catch_rate: 255,
            chance: 128,
        }],
        rng,
    )
}

fn active(state: &GameState) -> SafariSession {
    safari::session(state)
        .expect("valid session")
        .expect("active")
}

fn live_encounter(state: &GameState) -> SafariEncounter {
    match active(state).phase {
        SafariPhase::Encounter(encounter) | SafariPhase::Presenting { encounter, .. } => encounter,
        phase => panic!("expected wildlife, found {phase:?}"),
    }
}

fn binary_continue(state: GameState) -> GameState {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let directory = std::env::temp_dir().join(format!(
        "geothite-safari-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&directory).expect("unique test directory");
    let path = directory.join("session.crystalsave");
    let identity =
        SaveModpackIdentity::new("safari-regression", "01".repeat(32)).expect("identity");
    let content_hash = "02".repeat(32);
    save::write_save_game_for_modpack(&path, state, &identity, &content_hash)
        .expect("native binary save");
    let bytes = std::fs::read(&path).expect("saved bytes");
    let loaded = save::read_save_game_bytes_for_modpack(
        &bytes,
        "Safari regression",
        &identity,
        &content_hash,
    )
    .expect("Continue actual binary save");
    std::fs::remove_dir_all(directory).expect("remove test scratch");
    loaded.into_state()
}

#[test]
fn safari_departure_requires_real_arrival_and_cannot_discard_capture() {
    let mut state = exploring();
    let before = state.clone();
    let tile = TilePosition::new(16, 0);
    assert!(
        safari::complete_departure(&mut state, "MnPreservePineEntry", tile, Direction::Down)
            .is_err()
    );
    assert_eq!(state, before);
    state.overworld = OverworldMemory::Active {
        map_name: "MnPreservePineEntry".into(),
        tile,
        facing: Direction::Down,
        mode: MovementMode::Normal,
    };
    let storage = state.storage.clone();
    let bag = state.bag.clone();
    let money = state.money;
    safari::complete_departure(&mut state, "MnPreservePineEntry", tile, Direction::Down)
        .expect("committed native departure");
    assert!(safari::session(&state).expect("valid").is_none());
    assert_eq!(state.storage, storage);
    assert_eq!(state.bag, bag);
    assert_eq!(state.money, money);
    let mut state = capturing();
    let before = state.clone();
    assert!(
        safari::complete_departure(
            &mut state,
            "MnBoundarySafariC00R00",
            TilePosition::new(2, 2),
            Direction::Down
        )
        .is_err()
    );
    assert_eq!(state, before);
}

#[test]
fn safari_saved_cursor_and_ranger_choice_keep_input_ownership_after_continue() {
    let mut state = capturing();
    assert_eq!(
        safari::navigate_commands(&mut state, Direction::Up).expect("top bound"),
        0
    );
    safari::navigate_commands(&mut state, Direction::Down).expect("ROCK row");
    safari::navigate_commands(&mut state, Direction::Right).expect("RUN column");
    let mut state = binary_continue(state);
    assert_eq!(
        safari::selected_action(&state).expect("saved selection"),
        SafariAction::Run
    );
    action(&mut state, SafariAction::Run, &mut Draws::default()).expect("selected native RUN");
    assert!(
        safari::navigate_commands(&mut state, Direction::Left).is_err(),
        "pending result owns cursor"
    );
    let state = binary_continue(state);
    assert!(matches!(
        active(&state).phase,
        SafariPhase::Presenting { .. }
    ));

    let mut state = GameState::reset_wram_for_new_game();
    let prompt = crate::systems::safari_prompt::SafariPrompt {
        version: 1,
        zone_id: "boundary_waters".into(),
        map_name: "MnBoundaryWaters".into(),
        tile: TilePosition::new(30, 10),
        facing: Direction::Up,
        kind: crate::systems::safari_prompt::SafariPromptKind::Admission,
        selected_yes: true,
    };
    crate::systems::safari_prompt::set_prompt(&mut state, &prompt).expect("native offer");
    crate::systems::safari_prompt::navigate(&mut state, Direction::Down).expect("NO");
    let mut state = binary_continue(state);
    assert!(
        !crate::systems::safari_prompt::prompt(&state)
            .expect("valid")
            .expect("saved offer")
            .selected_yes
    );
    crate::systems::safari_prompt::clear_prompt(&mut state);
    assert!(
        crate::systems::safari_prompt::prompt(&state)
            .expect("valid")
            .is_none()
    );
}

#[test]
fn safari_bait_rock_duration_thresholds_and_draw_order() {
    let mut state = capturing();
    let before = state.clone();
    let mut rng = Draws::new(&[3, 32]);
    assert_eq!(
        action(&mut state, SafariAction::Bait, &mut rng)
            .expect("bait")
            .resolution,
        SafariResolution::Continue
    );
    assert_eq!(rng.carries, [false, false]);
    assert!(rng.bytes.is_empty());
    assert_eq!(
        live_encounter(&state).disposition,
        SafariDisposition::Eating {
            responses_remaining: 4
        }
    );
    assert_eq!(active(&state).balls_remaining, 30);
    assert_eq!(state.storage, before.storage);
    assert_eq!(state.party, before.party);
    assert_eq!(state.bag, before.bag);
    assert_eq!(state.money, before.money);
    assert_eq!(state.pokedex, before.pokedex);
    safari::acknowledge_result(&mut state).expect("present bait");
    // Rock replaces the four remaining eating responses; 127 < 128 flees.
    let mut rng = Draws::new(&[0, 127]);
    assert_eq!(
        action(&mut state, SafariAction::Rock, &mut rng)
            .expect("rock")
            .resolution,
        SafariResolution::Fled
    );
    assert_eq!(
        live_encounter(&state).disposition,
        SafariDisposition::Angry {
            responses_remaining: 1
        }
    );
    assert_eq!(rng.carries, [false, false]);
    safari::acknowledge_result(&mut state).expect("present flee");
    assert_eq!(active(&state).phase, SafariPhase::Exploring);
}

#[test]
fn safari_ball_reuses_native_full_hp_rate_and_expires_disposition() {
    let mut state = capturing();
    action(&mut state, SafariAction::Bait, &mut Draws::new(&[0, 255])).expect("two-response bait");
    safari::acknowledge_result(&mut state).expect("present bait");
    let mut rng = Draws::new(&[255, 255, 32]);
    let result = action(&mut state, SafariAction::Ball, &mut rng).expect("failed ball");
    let capture = result.capture.expect("native catch roll");
    assert!(!capture.caught);
    assert_eq!(capture.final_catch_rate, 30); // 120 × eating 1/2 × ball 3/2 ÷ 3.
    assert_eq!(capture.wobble_count, 0);
    assert_eq!(result.resolution, SafariResolution::Continue);
    assert_eq!(rng.carries, [false, false, false]); // catch, first failed wobble, flee.
    assert_eq!(
        live_encounter(&state).disposition,
        SafariDisposition::Neutral
    );
    assert_eq!(active(&state).balls_remaining, 29);
    safari::acknowledge_result(&mut state).expect("present failed ball");
    let result = action(
        &mut state,
        SafariAction::Ball,
        &mut Draws::new(&[255, 0, 0, 255, 64]),
    )
    .expect("neutral failure");
    assert_eq!(result.capture.expect("capture").final_catch_rate, 60);
    assert_eq!(result.resolution, SafariResolution::Continue);
}

#[test]
fn safari_rock_refreshes_clamps_and_does_not_stack() {
    assert_eq!(
        SafariDisposition::Eating {
            responses_remaining: 1
        }
        .scale(1),
        1
    );
    assert_eq!(
        SafariDisposition::Angry {
            responses_remaining: 1
        }
        .scale(200),
        255
    );
    let mut state = capturing();
    for _ in 0..3 {
        action(&mut state, SafariAction::Rock, &mut Draws::new(&[0, 128]))
            .expect("rock threshold equality survives");
        assert_eq!(
            live_encounter(&state).disposition,
            SafariDisposition::Angry {
                responses_remaining: 1
            }
        );
        safari::acknowledge_result(&mut state).expect("present rock");
    }
    let result = action(
        &mut state,
        SafariAction::Ball,
        &mut Draws::new(&[255, 255, 128]),
    )
    .expect("angry throw");
    assert_eq!(result.capture.expect("native capture").final_catch_rate, 85);
    assert_eq!(result.resolution, SafariResolution::Continue);
    assert_eq!(
        live_encounter(&state).disposition,
        SafariDisposition::Neutral
    );
}

#[test]
fn safari_last_ball_catch_storage_pokedex_and_pending_return_survive_binary_continue() {
    for to_pc in [false, true] {
        let mut state = exploring();
        if to_pc {
            for _ in 0..PARTY_SIZE {
                assert!(state.storage.party.add_pokemon(wildlife()));
            }
            state.sync_party_from_storage();
            state.current_pc_box = 2;
        }
        let mut session = active(&state);
        session.balls_remaining = 1;
        safari::set_session(&mut state, &session).expect("last-ball fixture");
        safari::begin_encounter(&mut state, encounter()).expect("wildlife");
        let before = state.clone();
        let mut rng = Draws::new(&[0]);
        let result = action(&mut state, SafariAction::Ball, &mut rng).expect("actual catch");
        assert_eq!(rng.carries, [false]); // No flee roll follows a catch.
        assert!(rng.bytes.is_empty());
        let SafariResolution::Caught(stored) = result.resolution else {
            panic!("catch must be delivered");
        };
        assert_eq!(
            stored.location,
            if to_pc {
                CaptureStorageLocation::Pc {
                    box_index: 2,
                    slot: 0,
                }
            } else {
                CaptureStorageLocation::Party { slot: 0 }
            }
        );
        assert_eq!(
            stored
                .pokemon
                .caught_data
                .as_ref()
                .expect("caught data")
                .location,
            42
        );
        assert_eq!(stored.pokemon.original_trainer_id, state.player_id);
        assert!(state.pokedex.has_caught("GILLPUP"));
        assert_eq!(state.bag, before.bag);
        assert_eq!(state.money, before.money);
        assert_eq!(active(&state).balls_remaining, 0);
        assert!(matches!(
            active(&state).phase,
            SafariPhase::Presenting { .. }
        ));
        assert_eq!(
            active(&state).pending_end,
            Some(SafariEndReason::BallsExhausted)
        );
        let caught = state.clone();
        assert!(action(&mut state, SafariAction::Ball, &mut Draws::default()).is_err());
        assert_eq!(state, caught);
        let mut state = binary_continue(state);
        assert_eq!(active(&state), active(&caught));
        assert_eq!(state.storage, caught.storage);
        safari::acknowledge_result(&mut state).expect("present saved catch once");
        assert_eq!(
            active(&state).phase,
            SafariPhase::Ending(SafariEndReason::BallsExhausted)
        );
        assert!(safari::complete_return(&mut state).is_err());
        let return_to = active(&state).return_to;
        // Component setup of an already committed native arrival, not warp proof.
        state.overworld = OverworldMemory::Active {
            map_name: return_to.map_name,
            tile: return_to.tile,
            facing: return_to.facing,
            mode: MovementMode::Normal,
        };
        safari::complete_return(&mut state).expect("clear only after matching return");
        assert!(safari::session(&state).expect("session").is_none());
        assert!(safari::complete_return(&mut state).is_err());
        assert_eq!(state.storage, caught.storage);
    }
}

#[test]
fn safari_full_current_storage_and_bad_capture_rules_change_nothing_before_rng() {
    let mut state = capturing();
    for _ in 0..PARTY_SIZE {
        assert!(state.storage.party.add_pokemon(wildlife()));
    }
    for _ in 0..MAX_BOX_MONS {
        state
            .storage
            .register_capture_in_box(0, wildlife())
            .expect("fill current PC");
    }
    state.sync_party_from_storage();
    assert!(
        state
            .storage
            .has_capture_space_in_box(1)
            .expect("other box space")
    );
    let before = state.clone();
    let mut rng = Draws::default();
    assert_eq!(
        action(&mut state, SafariAction::Ball, &mut rng),
        Err(SafariError::StorageFull)
    );
    assert!(rng.carries.is_empty());
    assert_eq!(state, before);
    assert_eq!(
        action(&mut state, SafariAction::Run, &mut rng)
            .expect("full storage can run")
            .resolution,
        SafariResolution::Ran
    );
    assert!(rng.carries.is_empty());
    let mut state = capturing();
    let before = state.clone();
    assert!(
        safari_encounter::resolve_action(
            &mut state,
            SafariAction::Ball,
            &CaptureRules::default(),
            &[],
            &mut rng
        )
        .is_err()
    );
    assert_eq!(state, before);
    assert!(rng.carries.is_empty());
    assert!(
        safari_encounter::resolve_action(&mut state, SafariAction::Ball, &rules(), &[], &mut rng)
            .is_err()
    );
    assert_eq!(state, before);
    assert!(rng.carries.is_empty());
}

#[test]
fn safari_successful_commits_exhaust_once_and_inactive_state_stays_byte_identical() {
    let mut inactive = GameState::reset_wram_for_new_game();
    let before = inactive.clone();
    assert_eq!(
        safari::record_successful_travel(&mut inactive).expect("ordinary travel"),
        None
    );
    assert_eq!(inactive, before);
    let mut state = exploring();
    for remaining in (1..500).rev() {
        assert_eq!(
            safari::record_successful_travel(&mut state).expect("one committed travel"),
            None
        );
        assert_eq!(active(&state).steps_remaining, remaining);
    }
    let mut state = binary_continue(state);
    assert_eq!(active(&state).steps_remaining, 1);
    assert_eq!(
        safari::record_successful_travel(&mut state).expect("last travel lands first"),
        Some(SafariEndReason::StepsExhausted)
    );
    let before = state.clone();
    assert!(safari::begin_encounter(&mut state, encounter()).is_err());
    assert!(safari::record_successful_travel(&mut state).is_err());
    assert_eq!(state, before);
}

#[test]
fn safari_pending_exhaustion_defers_retirement_and_failed_throw_until_result_ack() {
    let mut state = exploring();
    let mut session = active(&state);
    session.balls_remaining = 1;
    safari::set_session(&mut state, &session).expect("last ball");
    safari::begin_encounter(&mut state, encounter()).expect("wildlife");
    let result = action(
        &mut state,
        SafariAction::Ball,
        &mut Draws::new(&[255, 255, 255]),
    )
    .expect("failed throw, remains");
    assert_eq!(result.resolution, SafariResolution::Continue);
    safari::request_end(&mut state, SafariEndReason::Retired).expect("keep existing exhaustion");
    assert_eq!(
        active(&state).pending_end,
        Some(SafariEndReason::BallsExhausted)
    );
    assert!(matches!(
        active(&state).phase,
        SafariPhase::Presenting { .. }
    ));
    safari::acknowledge_result(&mut state).expect("visible result released");
    assert_eq!(
        active(&state).phase,
        SafariPhase::Ending(SafariEndReason::BallsExhausted)
    );
    let mut state = capturing();
    action(&mut state, SafariAction::Run, &mut Draws::default()).expect("run");
    safari::request_end(&mut state, SafariEndReason::LeftPreserve)
        .expect("defer outward transition");
    assert!(matches!(
        active(&state).phase,
        SafariPhase::Presenting { .. }
    ));
    safari::acknowledge_result(&mut state).expect("release run");
    assert_eq!(
        active(&state).phase,
        SafariPhase::Ending(SafariEndReason::LeftPreserve)
    );
}

#[test]
fn safari_corrupted_session_is_rejected_instead_of_resetting_budgets() {
    let state = capturing();
    for (field, value) in [
        ("version", serde_json::json!(2)),
        ("balls_remaining", serde_json::json!(31)),
        ("steps_remaining", serde_json::json!(501)),
        ("zone_id", serde_json::json!("invalid zone")),
        ("pending_end", serde_json::json!("steps_exhausted")),
    ] {
        let mut damaged = state.clone();
        let mut encoded = serde_json::to_value(active(&state)).expect("session JSON");
        encoded[field] = value;
        damaged
            .script_runtime
            .memory
            .insert(super::SESSION_KEY.into(), encoded.to_string());
        assert!(damaged.validate_saved_state().is_err(), "reject {field}");
        assert!(safari::session(&damaged).is_err());
    }
    let mut damaged = state.clone();
    damaged
        .script_runtime
        .memory
        .insert(super::SESSION_KEY.into(), "not JSON".into());
    assert!(damaged.validate_saved_state().is_err());
    let mut damaged = state;
    damaged.bug_contest.timer_active = true;
    assert!(safari::validate_saved_session(&damaged).is_err());
}

#[test]
fn safari_legacy_binary_baseline() {
    use sha2::{Digest, Sha256};
    let mut state = GameState::reset_wram_for_new_game();
    state
        .script_runtime
        .memory
        .insert("wSafariBalls".into(), "7".into());
    state
        .script_runtime
        .memory
        .insert("wSafariSteps".into(), "123".into());
    // This golden describes v25, before the independently versioned daycare
    // field. Verify its original wire and checksum rather than v26's new shape.
    let (checksum, bytes) = save::with_v25_day_care_wire(|| {
        let checksum = multiplayer::game_state_checksum_unchecked(&state).expect("checksum");
        let bytes = bincode::serde::encode_to_vec(
            &state,
            bincode::config::standard()
                .with_little_endian()
                .with_fixed_int_encoding(),
        )
        .expect("binary state");
        (checksum, bytes)
    });
    assert_eq!(checksum.hash(), 1_987_639_539);
    assert_eq!(bytes.len(), 7135);
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        "b4791b9ca12d4039e139c45d7515fda91a15ad92b6f865ecfd828b76e794c281"
    );
    assert!(safari::session(&state).expect("no paid permit").is_none());
}

#[test]
#[ignore = "requires CRYSTAL_SAFARI_LEGACY_SAVE produced before Safari support"]
fn safari_preexisting_binary_save_retains_checksum_and_exact_bytes() {
    let path = std::env::var_os("CRYSTAL_SAFARI_LEGACY_SAVE").expect("existing ignored save path");
    let bytes = std::fs::read(path).expect("read preexisting save, never regenerate this input");
    assert!(bytes.starts_with(b"CRYSTALSAVE\0"));
    assert_eq!(u16::from_be_bytes([bytes[12], bytes[13]]), 25);
    let (decoded, consumed): (save::SaveGame, usize) = bincode::serde::decode_from_slice(
        &bytes[22..],
        bincode::config::standard()
            .with_little_endian()
            .with_fixed_int_encoding(),
    )
    .expect("legacy save and its original authoritative checksum remain valid");
    assert_eq!(consumed, bytes.len() - 22);
    assert!(
        safari::session(decoded.state())
            .expect("old save is inactive")
            .is_none()
    );
    let loaded = save::read_save_game_bytes_for_modpack(
        &bytes,
        "preexisting native save",
        decoded.metadata().modpack(),
        decoded.metadata().pack_content_hash(),
    )
    .expect("native reader checks envelope, original state hash and identity");
    assert_eq!(
        save::encode_save_game_bytes(&loaded).expect("re-encode without normalization"),
        bytes
    );
}
