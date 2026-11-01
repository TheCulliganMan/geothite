//! Actual external preserve/config fixtures; no production controller claim.

#[path = "safari_departure_tests.rs"]
mod departure;
#[path = "safari_fishing_tests.rs"]
mod fishing;
#[path = "safari_generation_tests.rs"]
mod generation;

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove, MAX_BOX_MONS, PARTY_SIZE};
use crystal_core::random::RecordingDivider;
use crystal_core::random::ReplayDivider;
use crystal_core::state::GameState;
use crystal_core::systems::map_context::SpawnMemoryUpdate;
use crystal_core::systems::safari::{
    self, SafariDisposition, SafariEncounter, SafariEndReason, SafariPhase,
};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::session::OverworldSession;

use super::{SafariZone, collision};
use crate::{
    CompiledGamePack, GameDataSet, SafariAdmissionOutcome, read_verified_compiled_game_pack,
};

fn base() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        read_verified_compiled_game_pack(PathBuf::from(
            std::env::var_os("CRYSTAL_SAFARI_TEST_PACK")
                .expect("selected completed Minnesota pack"),
        ))
        .expect(
            "old pack still passes exact identity verification with absent optional Safari config",
        )
    })
}

fn config() -> SafariZone {
    serde_json::from_str(
        &std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../planning/minnesota/revisions/boundary-safari-runtime-v01/zone.json"),
        )
        .expect("authored regional Safari config"),
    )
    .expect("typed Safari config")
}

fn configured() -> GameDataSet {
    let mut data = base().data.clone();
    data.safari_zone = Some(config());
    data
}

fn ranger_fixture(data: &GameDataSet) -> (GameState, OverworldSession) {
    let zone = data.safari_zone.as_ref().expect("config");
    let mut state = GameState::reset_wram_for_new_game();
    state.player_name = "CHRIS".into();
    state.money = 1234;
    let mut partner = data
        .create_pokemon("GILLPUP", 35, Dv::from_non_hp(10, 10, 10, 10))
        .expect("native regional Surf partner");
    partner.moves = vec![LearnedMove {
        name: "SURF".into(),
        current_pp: data.moves["SURF"].pp,
        pp_ups: 0,
    }];
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.badges.johto[3] = true;
    let mut overworld = data
        .overworld_session(&zone.admission_map, zone.return_to.tile, 0)
        .expect("actual free hub");
    overworld.player.facing = zone.return_to.facing;
    overworld.sync_event_flag_memory(&state.flags);
    data.commit_overworld_snapshot(&mut state, &overworld, SpawnMemoryUpdate::Preserve);
    (state, overworld)
}

fn unchanged(
    state: &GameState,
    map: &OverworldSession,
    before: &(GameState, crystal_core::world::session::OverworldSnapshot),
) {
    assert_eq!(state, &before.0);
    assert_eq!(map.snapshot(), before.1);
}

#[test]
#[ignore = "requires CRYSTAL_SAFARI_TEST_PACK with expanded native preserve"]
fn safari_zone_matches_native_preserve_habitats_and_rejects_invalid_sources() {
    let data = &base().data;
    assert!(
        data.safari_zone.is_none(),
        "do not activate an unfinished live feature"
    );
    let zone = config();
    zone.validate(data)
        .expect("actual source, native coordinates, all species/evolution levels");
    assert_eq!(zone.maps.len(), 7);
    assert_eq!(zone.habitats.len(), 4);
    assert!(
        zone.habitat_at(data, "MnBoundaryRootBeerCabin", TilePosition::new(3, 5))
            .expect("included cabin")
            .is_none()
    );
    assert!(
        zone.habitat_at(data, "MnBoundarySafariC00R02", TilePosition::new(15, 9))
            .expect("entry court")
            .is_none()
    );
    let mut bad = zone.clone();
    bad.habitats[0].grass[0].weight -= 1;
    assert!(bad.validate(data).is_err());
    let mut bad = zone.clone();
    bad.habitats[0].grass[0].species = "MISSING_POKEMON".into();
    assert!(bad.validate(data).is_err());
    for species in ["WOOPER", "MORLIT"] {
        let mut bad = zone.clone();
        bad.habitats[0].grass[0].species = species.into();
        assert!(
            bad.validate(data).is_err(),
            "non-Minnesota and retired wildlife cannot enter a native habitat"
        );
    }
    let mut bad = zone.clone();
    bad.habitats[0]
        .grass
        .iter_mut()
        .find(|slot| slot.species == "MOSQUITAN")
        .expect("rare mosquito")
        .level = 54;
    assert!(
        bad.validate(data).is_err(),
        "native Mosquitan requires level55"
    );
    let mut bad = zone.clone();
    bad.entry.tile = TilePosition::new(-1, 9);
    assert!(bad.validate(data).is_err());
    let mut bad = zone.clone();
    bad.habitats[0].bounds_core = [[0, 0], [128, 112]];
    assert!(bad.validate(data).is_err());
    let mut missing = data.clone();
    missing.maps.remove("MnBoundarySafariC01R00");
    assert!(
        zone.validate(&missing).is_err(),
        "missing seam target is recoverable, never an indexing panic"
    );
    let mut outward = data.clone();
    let warp = &mut outward
        .maps
        .get_mut("MnBoundaryRootBeerCabin")
        .expect("cabin")
        .events
        .warps[0];
    warp.target_map = zone.admission_map.clone();
    assert!(
        zone.validate(&outward).is_err(),
        "only explicit return transactions leave preserve"
    );
}

#[test]
#[ignore = "requires CRYSTAL_SAFARI_TEST_PACK with expanded native preserve"]
fn safari_native_admission_refusals_and_failed_warp_are_atomic() {
    let data = configured();
    let (mut state, mut map) = ranger_fixture(&data);
    let original = (state.clone(), map.snapshot());
    assert_eq!(
        data.admit_safari(&mut state, &mut map, false, &BTreeSet::new())
            .expect("decline"),
        SafariAdmissionOutcome::Declined
    );
    unchanged(&state, &map, &original);
    state.badges.johto[3] = false;
    let before = (state.clone(), map.snapshot());
    assert_eq!(
        data.admit_safari(&mut state, &mut map, true, &BTreeSet::new())
            .expect("Surf badge refusal"),
        SafariAdmissionOutcome::NeedSurf
    );
    unchanged(&state, &map, &before);
    state.badges.johto[3] = true;
    state.money = 499;
    let before = (state.clone(), map.snapshot());
    assert_eq!(
        data.admit_safari(&mut state, &mut map, true, &BTreeSet::new())
            .expect("money refusal"),
        SafariAdmissionOutcome::InsufficientMoney
    );
    unchanged(&state, &map, &before);
    state.money = 1234;
    let before = (state.clone(), map.snapshot());
    assert!(
        data.admit_safari(
            &mut state,
            &mut map,
            true,
            &BTreeSet::from(["MUSIC_MISSING".into()])
        )
        .is_err(),
        "native map-music failure rolls back debit and warp"
    );
    unchanged(&state, &map, &before);
    let filler = state.storage.party.pokemon[0]
        .as_ref()
        .expect("Surf partner")
        .clone();
    for _ in 1..PARTY_SIZE {
        assert!(state.storage.party.add_pokemon(filler.clone()));
    }
    for _ in 0..MAX_BOX_MONS {
        state
            .storage
            .register_capture_in_box(0, filler.clone())
            .expect("full current box");
    }
    state.sync_party_from_storage();
    let before = (state.clone(), map.snapshot());
    assert_eq!(
        data.admit_safari(&mut state, &mut map, true, &BTreeSet::new())
            .expect("storage refusal"),
        SafariAdmissionOutcome::StorageFull
    );
    unchanged(&state, &map, &before);
}

#[test]
#[ignore = "requires CRYSTAL_SAFARI_TEST_PACK with expanded native preserve"]
fn safari_native_admission_and_return_commit_real_map_once_and_keep_party() {
    let data = configured();
    let (mut state, mut map) = ranger_fixture(&data);
    let original_party = state.storage.clone();
    assert_eq!(
        data.admit_safari(&mut state, &mut map, true, &BTreeSet::new())
            .expect("native staged admission"),
        SafariAdmissionOutcome::Admitted
    );
    assert_eq!(state.money, 734);
    let zone = data.safari_zone.as_ref().expect("config");
    assert_eq!(map.map.name, zone.entry.map_name);
    assert_eq!(map.player.tile, zone.entry.tile);
    assert_eq!(map.player.facing, zone.entry.facing);
    assert_eq!(
        collision(&data, &map.map.name, map.player.tile).expect("entry collision"),
        "FLOOR"
    );
    assert_eq!(state.storage, original_party);
    let active = safari::session(&state).expect("valid").expect("admitted");
    assert_eq!((active.balls_remaining, active.steps_remaining), (30, 500));
    data.validate_saved_safari_references(&state)
        .expect("actual native references");
    let before = (state.clone(), map.snapshot());
    assert_eq!(
        data.admit_safari(&mut state, &mut map, true, &BTreeSet::new())
            .expect("duplicate confirmation"),
        SafariAdmissionOutcome::AlreadyActive
    );
    unchanged(&state, &map, &before);
    assert!(
        base()
            .data
            .validate_saved_safari_references(&state)
            .is_err(),
        "old pack cannot silently reset an active session"
    );
    assert!(
        data.return_from_safari(&mut state, &mut map, &BTreeSet::new())
            .is_err(),
        "live expedition must settle first"
    );
    safari::request_end(&mut state, SafariEndReason::Retired).expect("retire");
    let before = (state.clone(), map.snapshot());
    assert!(
        data.return_from_safari(
            &mut state,
            &mut map,
            &BTreeSet::from(["MUSIC_MISSING".into()])
        )
        .is_err()
    );
    unchanged(&state, &map, &before);
    data.return_from_safari(&mut state, &mut map, &BTreeSet::new())
        .expect("actual safe native return");
    assert_eq!(map.map.name, zone.return_to.map_name);
    assert_eq!(map.player.tile, zone.return_to.tile);
    assert!(safari::session(&state).expect("valid").is_none());
    assert_eq!(state.money, 734);
    assert_eq!(state.storage, original_party);
}

#[test]
#[ignore = "requires CRYSTAL_SAFARI_TEST_PACK with expanded native preserve"]
fn safari_saved_wildlife_requires_real_habitat_surface_species_level_and_permit() {
    let data = configured();
    let (mut state, mut map) = ranger_fixture(&data);
    data.admit_safari(&mut state, &mut map, true, &BTreeSet::new())
        .expect("admission");
    let zone = data.safari_zone.as_ref().expect("config");
    // Explicit component origin fixture; production encounter travel is separate.
    let name = "MnBoundarySafariC00R02";
    let target = (0..16)
        .flat_map(|y| (0..64).map(move |x| TilePosition::new(x, y)))
        .find(|tile| collision(&data, name, *tile).is_ok_and(|terrain| terrain == "TALL_GRASS"))
        .expect("real grassy portage");
    map = data
        .overworld_session(name, target, 0)
        .expect("native habitat fixture");
    data.commit_overworld_snapshot(&mut state, &map, SpawnMemoryUpdate::Preserve);
    let habitat = zone
        .habitat_at(&data, name, target)
        .expect("habitat")
        .expect("exterior");
    let slot = &habitat.grass[0];
    let encounter = SafariEncounter {
        pokemon: data
            .create_pokemon(&slot.species, slot.level, Dv::from_non_hp(10, 10, 10, 10))
            .expect("real wild build"),
        habitat_id: habitat.id.clone(),
        origin_map_name: name.into(),
        origin_tile: target,
        caught_location: data.safari_caught_location(name).expect("native landmark"),
        base_flee_threshold: slot.flee_threshold,
        disposition: SafariDisposition::Neutral,
        resolved_actions: 0,
        source: Default::default(),
    };
    safari::begin_encounter(&mut state, encounter).expect("typed native wildlife");
    data.validate_saved_safari_references(&state)
        .expect("exact native origin");
    let mut bad_location = state.clone();
    let mut saved = safari::session(&bad_location)
        .expect("valid")
        .expect("permit");
    let SafariPhase::Encounter(wild) = &mut saved.phase else {
        panic!("wildlife");
    };
    assert_ne!(wild.caught_location, 0, "real preserve landmark is known");
    wild.caught_location = 0;
    safari::set_session(&mut bad_location, &saved).expect("structural fixture");
    assert!(
        data.validate_saved_safari_references(&bad_location)
            .is_err(),
        "the previous unknown-area fallback must be rejected"
    );
    let mut active = safari::session(&state).expect("valid").expect("active");
    let SafariPhase::Encounter(wild) = &mut active.phase else {
        panic!("encounter");
    };
    wild.habitat_id = "QUIET_WATER".into();
    safari::set_session(&mut state, &active).expect("structurally valid core");
    assert!(
        data.validate_saved_safari_references(&state).is_err(),
        "pack-specific geography cannot be fabricated"
    );
    let mut inactive = ranger_fixture(&data).0;
    data.commit_overworld_snapshot(&mut inactive, &map, SpawnMemoryUpdate::Preserve);
    assert!(
        data.validate_saved_safari_references(&inactive).is_err(),
        "stale arrivals cannot create a permit"
    );
}

fn native_press(
    data: &GameDataSet,
    state: &mut GameState,
    map: &mut OverworldSession,
    button: GameButton,
) {
    let mut divider = ReplayDivider::new(std::iter::repeat_n(17, 1024));
    data.apply_overworld_input(state, map, [button], &BTreeSet::new(), &mut divider)
        .expect("native committed input");
    for _ in 0..16 {
        data.apply_overworld_input(state, map, [], &BTreeSet::new(), &mut divider)
            .expect("native release and presentation frames");
    }
}

#[test]
#[ignore = "requires CRYSTAL_SAFARI_TEST_PACK with expanded native preserve"]
fn safari_existing_free_gate_court_blocks_without_permit_and_native_travel_counts_once() {
    let data = configured();
    let (mut state, mut map) = ranger_fixture(&data);
    let zone = data.safari_zone.as_ref().expect("config");
    let court = zone.gate_court.map_name.as_str();
    data.transition_overworld_session(
        &mut state,
        &mut map,
        court,
        TilePosition::new(12, 6),
        SpawnMemoryUpdate::Preserve,
        &BTreeSet::new(),
    )
    .expect("free existing walking court fixture");
    map.player.facing = Direction::Up;
    data.commit_overworld_snapshot(&mut state, &map, SpawnMemoryUpdate::Preserve);
    data.validate_saved_safari_references(&state)
        .expect("court is free");
    assert!(
        map.visible_object_at_checked(TilePosition::new(12, 5))
            .expect("native guard")
            .is_some()
    );
    native_press(&data, &mut state, &mut map, GameButton::Up);
    assert_eq!(
        map.player.tile,
        TilePosition::new(12, 6),
        "guard body seals lake access"
    );
    assert!(safari::session(&state).expect("valid").is_none());
    // Fresh explicit native component setup at the free ranger; admission is real.
    let (mut state, mut map) = ranger_fixture(&data);
    data.admit_safari(&mut state, &mut map, true, &BTreeSet::new())
        .expect("fee and actual entry");
    data.transition_overworld_session(
        &mut state,
        &mut map,
        court,
        TilePosition::new(12, 6),
        SpawnMemoryUpdate::Preserve,
        &BTreeSet::new(),
    )
    .expect("gate approach fixture");
    map.player.facing = Direction::Up;
    data.commit_overworld_snapshot(&mut state, &map, SpawnMemoryUpdate::Preserve);
    assert!(
        map.visible_object_at_checked(TilePosition::new(12, 5))
            .expect("native gate open")
            .is_none()
    );
    native_press(&data, &mut state, &mut map, GameButton::Up);
    assert_eq!(map.player.tile, TilePosition::new(12, 5));
    assert_eq!(
        safari::session(&state)
            .expect("valid")
            .expect("active")
            .steps_remaining,
        499
    );
    native_press(&data, &mut state, &mut map, GameButton::Right);
    assert_eq!(map.player.tile, TilePosition::new(12, 5), "turn costs zero");
    assert_eq!(
        safari::session(&state)
            .expect("valid")
            .expect("active")
            .steps_remaining,
        499
    );
    native_press(&data, &mut state, &mut map, GameButton::Up); // Face north again.
    native_press(&data, &mut state, &mut map, GameButton::Up);
    assert_eq!(map.player.tile, TilePosition::new(12, 4));
    assert_eq!(
        safari::session(&state)
            .expect("valid")
            .expect("active")
            .steps_remaining,
        498
    );
    let (mut state, mut map) = data
        .resume_overworld_session_from_state(
            state,
            &data
                .audio
                .iter()
                .filter(|asset| asset.kind == crate::ModpackAudioKind::Music)
                .map(|asset| asset.id.clone())
                .collect(),
        )
        .expect("native resume keeps permit and visibility");
    assert!(
        map.visible_object_at_checked(TilePosition::new(12, 5))
            .expect("resumed guard")
            .is_none()
    );
    let mut active = safari::session(&state).expect("valid").expect("active");
    active.steps_remaining = 1; // Explicit exhaustion fixture, not earned500-step proof.
    safari::set_session(&mut state, &active).expect("last-step setup");
    native_press(&data, &mut state, &mut map, GameButton::Up);
    assert_eq!(
        safari::session(&state)
            .expect("valid")
            .expect("active")
            .phase,
        SafariPhase::Ending(SafariEndReason::StepsExhausted)
    );
    let landing = map.player.tile;
    native_press(&data, &mut state, &mut map, GameButton::Down);
    assert_eq!(
        map.player.tile, landing,
        "ending cannot accept another travel"
    );
    data.return_from_safari(&mut state, &mut map, &BTreeSet::new())
        .expect("native exhausted return");
    data.transition_overworld_session(
        &mut state,
        &mut map,
        court,
        TilePosition::new(12, 6),
        SpawnMemoryUpdate::Preserve,
        &BTreeSet::new(),
    )
    .expect("free return visit");
    assert!(
        map.visible_object_at_checked(TilePosition::new(12, 5))
            .expect("closed again")
            .is_some()
    );
}

#[test]
#[ignore = "requires CRYSTAL_SAFARI_TEST_PACK with expanded native preserve"]
fn safari_native_mutation_replays_real_rng_and_rejects_bad_traces_atomically() {
    let data = configured();
    let (mut initial, mut map) = ranger_fixture(&data);
    data.admit_safari(&mut initial, &mut map, true, &BTreeSet::new())
        .expect("native admission");
    let name = "MnBoundarySafariC00R02";
    let tile = (0..16)
        .flat_map(|y| (0..64).map(move |x| TilePosition::new(x, y)))
        .find(|tile| collision(&data, name, *tile).is_ok_and(|terrain| terrain == "TALL_GRASS"))
        .expect("real portage");
    map = data
        .overworld_session(name, tile, 0)
        .expect("explicit component encounter origin");
    data.commit_overworld_snapshot(&mut initial, &map, SpawnMemoryUpdate::Preserve);
    let habitat = data
        .safari_zone
        .as_ref()
        .expect("config")
        .habitat_at(&data, name, tile)
        .expect("habitat")
        .expect("wild habitat");
    let slot = &habitat.grass[0];
    safari::begin_encounter(
        &mut initial,
        SafariEncounter {
            pokemon: data
                .create_pokemon(&slot.species, slot.level, Dv::default())
                .expect("native wildlife"),
            habitat_id: habitat.id.clone(),
            origin_map_name: name.into(),
            origin_tile: tile,
            caught_location: data.safari_caught_location(name).expect("native landmark"),
            base_flee_threshold: slot.flee_threshold,
            disposition: SafariDisposition::Neutral,
            resolved_actions: 0,
            source: Default::default(),
        },
    )
    .expect("explicit encounter fixture, not encounter-generation proof");
    let mut expected = initial.clone();
    let mut source = ReplayDivider::new(std::iter::repeat_n(17, 1024));
    let mut recording = RecordingDivider::new(&mut source);
    let action = crystal_core::systems::safari::SafariAction::Bait;
    let result = data
        .resolve_safari_action(&mut expected, action, &mut recording)
        .expect("native hardware RNG action");
    let trace = crate::RuntimeDividerTrace::new(recording.samples().iter().copied());
    assert!(!trace.samples.is_empty());
    assert_ne!(expected.random_state, initial.random_state);
    assert_eq!(expected.storage, initial.storage);
    assert_eq!(expected.bag, initial.bag);
    assert_eq!(expected.money, initial.money);
    let mut replayed = initial.clone();
    let mut replayed_map = map.clone();
    let outcome = data
        .apply_runtime_mutation_command(
            &mut replayed,
            &mut replayed_map,
            crate::RuntimeMutationCommand::Safari(crate::RuntimeSafariCommand::Act {
                action,
                divider_trace: trace.clone(),
            }),
            &BTreeSet::new(),
            &BTreeSet::new(),
            &BTreeSet::new(),
        )
        .expect("same production mutation used by runtime journal");
    assert_eq!(
        outcome.result,
        crate::RuntimeMutationResult::SafariUpdated(crate::RuntimeSafariOutcome::Action(result))
    );
    assert_eq!(replayed, expected);
    assert_eq!(replayed_map.snapshot(), map.snapshot());
    assert_eq!(
        outcome.state_checksum,
        crystal_core::multiplayer::game_state_checksum(&expected).expect("native state checksum")
    );
    let result_tile = replayed_map.player.tile;
    let result_budget = safari::session(&replayed)
        .expect("valid pending result")
        .expect("active")
        .steps_remaining;
    native_press(&data, &mut replayed, &mut replayed_map, GameButton::Down);
    assert_eq!(
        replayed_map.player.tile, result_tile,
        "capture presentation owns native movement input"
    );
    assert_eq!(
        safari::session(&replayed)
            .expect("valid")
            .expect("active")
            .steps_remaining,
        result_budget
    );
    for samples in [Vec::new(), {
        let mut extra = trace.samples.clone();
        extra.push(9);
        extra
    }] {
        let mut state = initial.clone();
        let mut native_map = map.clone();
        assert!(
            data.apply_runtime_mutation_command(
                &mut state,
                &mut native_map,
                crate::RuntimeMutationCommand::Safari(crate::RuntimeSafariCommand::Act {
                    action,
                    divider_trace: crate::RuntimeDividerTrace::new(samples)
                }),
                &BTreeSet::new(),
                &BTreeSet::new(),
                &BTreeSet::new()
            )
            .is_err()
        );
        unchanged(&state, &native_map, &(initial.clone(), map.snapshot()));
    }
}

#[test]
#[ignore = "requires CRYSTAL_SAFARI_TEST_PACK with CANOE"]
fn canoe_safari_admission_needs_no_surf_partner_and_still_charges_once() {
    let data = configured();
    let (mut state, mut map) = ranger_fixture(&data);
    for pokemon in state.storage.party.pokemon.iter_mut().flatten() {
        pokemon.moves.clear();
    }
    state.sync_party_from_storage();
    state.badges.johto[3] = false;
    assert!(
        state
            .bag
            .add_item(&data.items["CANOE"], 1)
            .expect("canoe owner")
    );
    let party = state.storage.clone();
    let money = state.money;
    assert_eq!(
        data.admit_safari(&mut state, &mut map, true, &BTreeSet::new())
            .expect("canoe admission"),
        SafariAdmissionOutcome::Admitted
    );
    assert_eq!(state.money, money - safari::ADMISSION_FEE);
    assert_eq!(state.storage, party);
    assert_eq!(
        data.admit_safari(&mut state, &mut map, true, &BTreeSet::new())
            .expect("repeat admission"),
        SafariAdmissionOutcome::AlreadyActive
    );
    assert_eq!(state.money, money - safari::ADMISSION_FEE);
}
